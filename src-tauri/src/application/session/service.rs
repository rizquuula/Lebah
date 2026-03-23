use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::application::event_bus::{DomainEvent, DomainEventBus};
use crate::application::ports::SessionManagerPort;
use crate::application::session::commands::*;
use crate::application::task::commands::{MarkTaskStartedCommand, MarkTaskStoppedCommand};
use crate::application::task::service::TaskApplicationService;
use crate::domain::agent::runner::{AgentHandle, AgentRunConfig, AgentRunner, PermissionMode};
use crate::domain::repositories::OutputRepository;
use crate::domain::project::value_objects::ProjectId;
use crate::domain::session::events::SessionDomainEvent;
use crate::domain::task::value_objects::TaskId;
use crate::infrastructure::agents::registry::AgentRegistry;

pub struct SessionApplicationService {
    agent_registry: Arc<AgentRegistry>,
    task_service: Arc<TaskApplicationService>,
    output_repo: Arc<dyn OutputRepository>,
    session_manager: Arc<dyn SessionManagerPort>,
    event_bus: Arc<dyn DomainEventBus>,
    current_project: Arc<std::sync::Mutex<Option<String>>>,
}

impl SessionApplicationService {
    pub fn new(
        agent_registry: Arc<AgentRegistry>,
        task_service: Arc<TaskApplicationService>,
        output_repo: Arc<dyn OutputRepository>,
        session_manager: Arc<dyn SessionManagerPort>,
        event_bus: Arc<dyn DomainEventBus>,
        current_project: Arc<std::sync::Mutex<Option<String>>>,
    ) -> Self {
        Self {
            agent_registry,
            task_service,
            output_repo,
            session_manager,
            event_bus,
            current_project,
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

        // Mark task as started
        let _ = self.task_service.clear_output(&cmd.task_id);
        self.task_service.mark_task_started(MarkTaskStartedCommand {
            id: cmd.task_id.clone(),
        })?;

        let run_config = AgentRunConfig {
            task_id: task_id.clone(),
            prompt: cmd.description,
            project_path: project_path.clone(),
            worktree: cmd.worktree,
            model: cmd.model,
            permission_mode: cmd.permission_mode,
            extra_args: Vec::new(),
            is_follow_up: false,
            agent_binary: cmd.agent_path,
        };

        let handle = runner.start(run_config).map_err(|e| {
            log::error!("[session] Failed to start session for task {}: {}", task_id.0, e);
            e
        })?;
        log::info!("[session] Session started successfully for task {}", task_id.0);
        let agent_name = runner.name().to_string();

        self.event_bus.publish(DomainEvent::Session(SessionDomainEvent::SessionStarted {
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
        self.task_service.mark_task_stopped(MarkTaskStoppedCommand {
            id: cmd.task_id,
        })?;
        Ok(())
    }

    pub fn send_input(&self, cmd: SendInputCommand) -> Result<(), ApplicationError> {
        log::info!(
            "[session] Sending input to task {}: input_len={} model={:?}",
            cmd.task_id, cmd.input.len(), cmd.model,
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
        self.event_bus.publish(DomainEvent::Session(SessionDomainEvent::SessionOutputReceived {
            task_id: task_id.clone(),
            line: user_input_line,
            project_path: pre_project_path,
        }));

        if let Some(ref m) = cmd.model {
            runner.update_model(&task_id, m)?;
        }

        let project_path = self.current_project_path()?;
        let run_config = AgentRunConfig {
            task_id: task_id.clone(),
            prompt: cmd.input,
            project_path: project_path.clone().map(crate::domain::project::value_objects::ProjectPath::new),
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
        };

        let handle = runner.send_follow_up(run_config).map_err(|e| {
            log::error!("[session] Failed to send follow-up for task {}: {}", task_id.0, e);
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
        let project_path = self.current_project_path().ok().flatten().unwrap_or_default();
        if project_path.is_empty() {
            return Vec::new();
        }
        let project_id = ProjectId::from_path(&project_path);
        self.output_repo.load_all(&project_id, &tid)
    }

    fn resolve_runner(&self, agent_name: Option<&str>) -> Result<Arc<dyn AgentRunner>, ApplicationError> {
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

    fn wire_handle(
        &self,
        handle: AgentHandle,
        task_id: TaskId,
        project_path: String,
    ) {
        log::debug!("[session] Wiring handle for task {}", task_id.0);
        let event_bus = Arc::clone(&self.event_bus);
        let task_id_c = task_id.clone();
        let pp = project_path.clone();

        // Wire stdout
        std::thread::spawn(move || {
            log::debug!("[session] Stdout reader thread started for task {}", task_id_c.0);
            for line in handle.stdout_rx {
                log::debug!("[claude-json] {}", line);
                event_bus.publish(DomainEvent::Session(SessionDomainEvent::SessionOutputReceived {
                    task_id: task_id_c.clone(),
                    line,
                    project_path: pp.clone(),
                }));
            }
        });

        let event_bus2 = Arc::clone(&self.event_bus);
        let task_id_c2 = task_id.clone();
        let pp2 = project_path.clone();

        // Wire stderr
        std::thread::spawn(move || {
            log::debug!("[session] Stderr reader thread started for task {}", task_id_c2.0);
            for line in handle.stderr_rx {
                event_bus2.publish(DomainEvent::Session(SessionDomainEvent::SessionOutputReceived {
                    task_id: task_id_c2.clone(),
                    line,
                    project_path: pp2.clone(),
                }));
            }
        });

        let event_bus3 = Arc::clone(&self.event_bus);
        let task_id_c3 = task_id;
        let pp3 = project_path;

        // Wire exit
        std::thread::spawn(move || {
            log::debug!("[session] Exit watcher thread started for task {}", task_id_c3.0);
            for success in handle.exit_rx {
                log::info!("[session] Task {} exited with success={}", task_id_c3.0, success);
                event_bus3.publish(DomainEvent::Session(SessionDomainEvent::SessionEnded {
                    task_id: task_id_c3.clone(),
                    success,
                    project_path: pp3.clone(),
                }));
                break;
            }
        });
    }
}
