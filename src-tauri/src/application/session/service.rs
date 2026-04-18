use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::application::event_bus::{DomainEvent, DomainEventBus};
use crate::application::ports::{SessionManagerPort, WorktreePort};
use crate::application::session::commands::*;
use crate::application::session::handle_pump::{pump_handle, PumpConfig, StreamKind};
use crate::application::task::commands::{
    MarkTaskCompletedCommand, MarkTaskStartedCommand, MarkTaskStoppedCommand,
};
use crate::application::task::service::TaskApplicationService;
use crate::domain::agent::runner::{AgentHandle, AgentRunConfig, AgentRunner, PermissionMode};
use crate::domain::project::value_objects::{ProjectId, ProjectPath};
use crate::domain::repositories::OutputRepository;
use crate::domain::session::events::SessionDomainEvent;
use crate::domain::task::value_objects::{TaskId, WorktreeRef};
use crate::infrastructure::agents::registry::AgentRegistry;

pub struct SessionApplicationService {
    agent_registry: Arc<AgentRegistry>,
    task_service: Arc<TaskApplicationService>,
    output_repo: Arc<dyn OutputRepository>,
    session_manager: Arc<dyn SessionManagerPort>,
    event_bus: Arc<dyn DomainEventBus>,
    current_project: Arc<std::sync::Mutex<Option<String>>>,
    worktree_port: Arc<dyn WorktreePort>,
}

impl SessionApplicationService {
    pub fn new(
        agent_registry: Arc<AgentRegistry>,
        task_service: Arc<TaskApplicationService>,
        output_repo: Arc<dyn OutputRepository>,
        session_manager: Arc<dyn SessionManagerPort>,
        event_bus: Arc<dyn DomainEventBus>,
        current_project: Arc<std::sync::Mutex<Option<String>>>,
        worktree_port: Arc<dyn WorktreePort>,
    ) -> Self {
        Self {
            agent_registry,
            task_service,
            output_repo,
            session_manager,
            event_bus,
            current_project,
            worktree_port,
        }
    }

    fn current_project_path(&self) -> Result<Option<String>, ApplicationError> {
        self.current_project
            .lock()
            .map(|g| g.clone())
            .map_err(|e| ApplicationError::Persistence(e.to_string()))
    }

    pub fn start_session(&self, cmd: StartSessionCommand) -> Result<(), ApplicationError> {
        log::info!(
            "[session] Starting session: task={} agent={:?} permission={:?} project={:?}",
            cmd.task_id,
            cmd.agent_name,
            cmd.permission_mode,
            cmd.project_path.as_ref().map(|p| p.as_str()),
        );
        let runner = self.resolve_runner(cmd.agent_name.as_deref())?;

        let task_id = TaskId::from_string(cmd.task_id.clone());
        let project_path = cmd.project_path.clone();

        // Capture before cmd is consumed by run_config
        let worktree_name = cmd.worktree.as_ref().map(|w| w.as_str().to_string());
        let worktree_links = cmd.worktree_links.clone();

        // Mark task as started
        let _ = self.task_service.clear_output(&cmd.task_id);
        self.task_service
            .mark_task_started(MarkTaskStartedCommand {
                id: cmd.task_id.clone(),
            })?;

        // If the agent doesn't natively support worktrees (e.g. OpenCode),
        // create the worktree manually and redirect the working directory.
        let mut effective_project_path = project_path.clone();
        let needs_manual_worktree =
            cmd.worktree.is_some() && !runner.capabilities().supports_worktree;

        if needs_manual_worktree {
            if let (Some(ref wt), Some(ref proj)) = (&cmd.worktree, &project_path) {
                match self.worktree_port.create(proj, wt) {
                    Ok(wt_dir) => {
                        log::info!(
                            "[session] Manual worktree created for non-native agent: {}",
                            wt_dir
                        );
                        effective_project_path = Some(ProjectPath::new(wt_dir));
                    }
                    Err(e) => {
                        log::error!("[session] Failed to create manual worktree: {}", e);
                        let _ = self.task_service.mark_task_completed(
                            MarkTaskCompletedCommand {
                                id: task_id.0.clone(),
                                success: false,
                            },
                            proj.as_str(),
                        );
                        return Err(e);
                    }
                }
            }
        }

        // Persist the initial prompt as a user_input event so the chat shows the
        // full template (not just task.description) in the bubble.
        let user_input_line = format!(
            r#"{{"type":"user_input","text":{}}}"#,
            serde_json::to_string(&cmd.description).unwrap_or_default()
        );
        let pre_project_path = project_path
            .as_ref()
            .map(|p| p.0.clone())
            .unwrap_or_default();
        self.event_bus.publish(DomainEvent::Session(
            SessionDomainEvent::SessionOutputReceived {
                task_id: task_id.clone(),
                line: user_input_line,
                project_path: pre_project_path,
            },
        ));

        let run_config = AgentRunConfig {
            task_id: task_id.clone(),
            prompt: cmd.description,
            project_path: effective_project_path.clone(),
            worktree: if needs_manual_worktree {
                None
            } else {
                cmd.worktree
            },
            model: cmd.model,
            permission_mode: cmd.permission_mode,
            extra_args: Vec::new(),
            is_follow_up: false,
            agent_binary: cmd.agent_path,
            env_vars: cmd.env_vars,
        };

        let handle = runner.start(run_config).map_err(|e| {
            log::error!(
                "[session] Failed to start session for task {}: {}",
                task_id.0,
                e
            );
            let project_path_str = project_path
                .as_ref()
                .map(|p| p.0.clone())
                .unwrap_or_default();
            let _ = self.task_service.mark_task_completed(
                MarkTaskCompletedCommand {
                    id: task_id.0.clone(),
                    success: false,
                },
                &project_path_str,
            );
            e
        })?;
        log::info!(
            "[session] Session started successfully for task {}",
            task_id.0
        );

        // Apply worktree links.
        // For agents with native worktree support (Claude): poll in background since
        // the CLI creates the worktree dir asynchronously.
        // For manual worktrees (OpenCode): dir already exists, apply immediately.
        if let (Some(wt_name), links) = (worktree_name, worktree_links) {
            if !links.is_empty() {
                if let Some(ref proj) = project_path {
                    let proj_str = proj.as_str().to_string();
                    let worktree_port = Arc::clone(&self.worktree_port);
                    if needs_manual_worktree {
                        // Worktree already exists — apply links synchronously
                        if let Err(e) = worktree_port.apply_links(
                            &ProjectPath::new(proj_str),
                            &WorktreeRef::new(wt_name),
                            &links,
                        ) {
                            log::warn!("[session] apply_links failed: {}", e);
                        }
                    } else {
                        std::thread::spawn(move || {
                            let wt_path = std::path::Path::new(&proj_str)
                                .join(".claude")
                                .join("worktrees")
                                .join(&wt_name);
                            let deadline =
                                std::time::Instant::now() + std::time::Duration::from_secs(30);
                            loop {
                                if wt_path.is_dir() {
                                    break;
                                }
                                if std::time::Instant::now() > deadline {
                                    log::warn!(
                                        "[session] Timed out waiting for worktree dir: {:?}",
                                        wt_path
                                    );
                                    return;
                                }
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            if let Err(e) = worktree_port.apply_links(
                                &ProjectPath::new(proj_str),
                                &WorktreeRef::new(wt_name),
                                &links,
                            ) {
                                log::warn!("[session] apply_links failed: {}", e);
                            }
                        });
                    }
                }
            }
        }
        let agent_name = runner.name().to_string();

        self.event_bus
            .publish(DomainEvent::Session(SessionDomainEvent::SessionStarted {
                task_id: task_id.clone(),
                agent_name,
            }));

        // Wire the AgentHandle into the event bus
        let project_path_str = project_path.map(|p| p.0).unwrap_or_default();
        self.wire_handle(handle, task_id, project_path_str);

        Ok(())
    }

    pub fn stop_session(&self, cmd: StopSessionCommand) -> Result<(), ApplicationError> {
        log::info!("[session] Stopping session {}", cmd.task_id);
        let runner = self.resolve_runner(None)?;
        let task_id = TaskId::from_string(cmd.task_id.clone());
        runner.terminate(&task_id)?;
        self.task_service
            .mark_task_stopped(MarkTaskStoppedCommand { id: cmd.task_id })?;
        Ok(())
    }

    pub fn send_input(&self, cmd: SendInputCommand) -> Result<(), ApplicationError> {
        log::info!(
            "[session] Sending input to task {}: input_len={} model={:?}",
            cmd.task_id,
            cmd.input.len(),
            cmd.model,
        );
        let runner = self.resolve_runner(None)?;
        let task_id = TaskId::from_string(cmd.task_id.clone());

        // Persist the user input as an output event so TerminalModal can show it
        // both in real-time (via tauri_event_emitter) and on re-open (via getOutputBuffer).
        let user_input_line = format!(
            r#"{{"type":"user_input","text":{}}}"#,
            serde_json::to_string(&cmd.input).unwrap_or_default()
        );
        let pre_project_path = self.current_project_path()?.unwrap_or_default();
        self.event_bus.publish(DomainEvent::Session(
            SessionDomainEvent::SessionOutputReceived {
                task_id: task_id.clone(),
                line: user_input_line,
                project_path: pre_project_path,
            },
        ));

        if let Some(ref m) = cmd.model {
            runner.update_model(&task_id, m)?;
        }

        let project_path = self.current_project_path()?;
        let run_config = AgentRunConfig {
            task_id: task_id.clone(),
            prompt: cmd.input,
            project_path: project_path
                .clone()
                .map(crate::domain::project::value_objects::ProjectPath::new),
            worktree: None,
            model: cmd.model,
            permission_mode: if cmd.yolo {
                PermissionMode::Sandbox
            } else {
                PermissionMode::Full
            },
            extra_args: Vec::new(),
            is_follow_up: true,
            agent_binary: None,
            env_vars: cmd.env_vars,
        };

        let handle = runner.send_follow_up(run_config).map_err(|e| {
            log::error!(
                "[session] Failed to send follow-up for task {}: {}",
                task_id.0,
                e
            );
            e
        })?;
        log::info!("[session] Follow-up started for task {}", task_id.0);
        let project_path_str = project_path.unwrap_or_default();
        self.wire_handle(handle, task_id, project_path_str);

        Ok(())
    }

    pub fn get_output_buffer(&self, task_id: &str) -> Vec<String> {
        let tid = TaskId::from_string(task_id.to_string());
        let live = self.session_manager.get_live_buffer(&tid);
        if !live.is_empty() {
            return live;
        }
        // Fall back to persisted output
        let project_path = self
            .current_project_path()
            .ok()
            .flatten()
            .unwrap_or_default();
        if project_path.is_empty() {
            return Vec::new();
        }
        let project_id = ProjectId::from_path(&project_path);
        self.output_repo.load_all(&project_id, &tid)
    }

    pub fn list_agents(&self) -> Vec<String> {
        self.agent_registry.list_runners()
    }

    fn resolve_runner(
        &self,
        agent_name: Option<&str>,
    ) -> Result<Arc<dyn AgentRunner>, ApplicationError> {
        let runner = match agent_name {
            Some(name) => {
                log::debug!("[session] Resolving agent runner: {}", name);
                self.agent_registry.get(name)
            }
            None => self.agent_registry.default_runner(),
        };
        if runner.is_none() {
            log::error!("[session] No agent runner found for {:?}", agent_name);
        }
        runner.ok_or_else(|| ApplicationError::NotFound("No agent runner available".to_string()))
    }

    fn wire_handle(&self, handle: AgentHandle, task_id: TaskId, project_path: String) {
        log::debug!("[session] Wiring handle for task {}", task_id.0);
        let event_bus = Arc::clone(&self.event_bus);
        let AgentHandle {
            stdout_rx,
            stderr_rx,
            exit_rx,
        } = handle;

        std::thread::spawn(move || {
            log::debug!("[session] Pump thread started for task {}", task_id.0);
            let task_id_out = task_id.clone();
            let pp_out = project_path.clone();
            let event_bus_out = Arc::clone(&event_bus);
            let task_id_end = task_id.clone();

            pump_handle(
                stdout_rx,
                stderr_rx,
                exit_rx,
                PumpConfig::default(),
                move |kind, line| {
                    match kind {
                        StreamKind::Stdout => log::debug!("[claude-json] {}", line),
                        StreamKind::Stderr => {
                            log::warn!("[session] stderr task={} {}", task_id_out.0, line)
                        }
                    }
                    event_bus_out.publish(DomainEvent::Session(
                        SessionDomainEvent::SessionOutputReceived {
                            task_id: task_id_out.clone(),
                            line,
                            project_path: pp_out.clone(),
                        },
                    ));
                },
                move |success| {
                    if success {
                        log::info!("[session] Task {} exited with success=true", task_id_end.0);
                    } else {
                        log::error!("[session] Task {} exited with success=false", task_id_end.0);
                    }
                    event_bus.publish(DomainEvent::Session(SessionDomainEvent::SessionEnded {
                        task_id: task_id_end,
                        success,
                        project_path,
                    }));
                },
            );
        });
    }
}
