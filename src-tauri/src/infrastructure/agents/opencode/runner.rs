use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::Child;
use std::sync::{Arc, Mutex};

use crate::domain::agent::runner::{
    AgentCapabilities, AgentError, AgentHandle, AgentRunConfig, AgentRunner,
};
use crate::domain::task::value_objects::TaskId;
use crate::infrastructure::agents::opencode::command_builder::OpencodeCommandBuilder;
use crate::infrastructure::agents::opencode::output_normalizer;

struct SessionConfig {
    config: AgentRunConfig,
    model_override: Option<String>,
}

pub struct OpencodeRunner {
    sessions: Arc<Mutex<HashMap<String, Child>>>,
    session_configs: Mutex<HashMap<String, SessionConfig>>,
}

impl OpencodeRunner {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_configs: Mutex::new(HashMap::new()),
        }
    }

    fn spawn_and_wire(&self, config: AgentRunConfig) -> Result<AgentHandle, AgentError> {
        let mut cmd = OpencodeCommandBuilder::build(&config);

        log::info!(
            "[opencode] Spawning: {} {:?}",
            cmd.get_program().to_string_lossy(),
            cmd.get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        );

        let mut child = cmd.spawn().map_err(|e| {
            log::error!(
                "[opencode] Failed to spawn process for task {}: {}",
                config.task_id.0,
                e
            );
            AgentError::SpawnFailed(format!("Failed to spawn opencode: {}", e))
        })?;
        log::info!(
            "[opencode] Process spawned for task {}, pid={:?}",
            config.task_id.0,
            child.id()
        );

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Raw stdout channel — will be normalized
        let (raw_stdout_tx, raw_stdout_rx) = std::sync::mpsc::channel();
        let (stderr_tx, stderr_rx) = std::sync::mpsc::channel();
        let (exit_tx, exit_rx) = std::sync::mpsc::channel();

        if let Some(stdout) = stdout {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    if raw_stdout_tx.send(line).is_err() {
                        break;
                    }
                }
            });
        }

        // Normalize opencode output to Claude-compatible format
        let stdout_rx = output_normalizer::spawn_normalizer(raw_stdout_rx);

        if let Some(stderr) = stderr {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    if stderr_tx.send(line).is_err() {
                        break;
                    }
                }
            });
        }

        // Exit monitor thread
        let task_id_str = config.task_id.0.clone();
        let sessions_arc = Arc::clone(&self.sessions);
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let mut sessions = match sessions_arc.lock() {
                Ok(s) => s,
                Err(_) => break,
            };
            if let Some(child) = sessions.get_mut(&task_id_str) {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        log::info!(
                            "[opencode] Process exited for task {}: status={}",
                            task_id_str,
                            status
                        );
                        sessions.remove(&task_id_str);
                        drop(sessions);
                        let _ = exit_tx.send(status.success());
                        break;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        log::error!(
                            "[opencode] Error polling process for task {}: {}",
                            task_id_str,
                            e
                        );
                        sessions.remove(&task_id_str);
                        drop(sessions);
                        let _ = exit_tx.send(false);
                        break;
                    }
                }
            } else {
                break;
            }
        });

        {
            let mut sessions = self
                .sessions
                .lock()
                .map_err(|e| AgentError::Internal(e.to_string()))?;
            sessions.insert(config.task_id.0.clone(), child);
        }

        {
            let mut configs = self
                .session_configs
                .lock()
                .map_err(|e| AgentError::Internal(e.to_string()))?;
            configs.insert(
                config.task_id.0.clone(),
                SessionConfig {
                    config,
                    model_override: None,
                },
            );
        }

        Ok(AgentHandle {
            stdout_rx,
            stderr_rx,
            exit_rx,
        })
    }
}

impl AgentRunner for OpencodeRunner {
    fn name(&self) -> &str {
        "opencode"
    }

    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            supports_plan_mode: false,
            supports_session_resume: true,
            supports_worktree: false,
            supports_model_selection: true,
            supports_follow_up: true,
        }
    }

    fn start(&self, config: AgentRunConfig) -> Result<AgentHandle, AgentError> {
        log::info!("[opencode] Starting session for task {}", config.task_id.0);
        {
            let sessions = self
                .sessions
                .lock()
                .map_err(|e| AgentError::Internal(e.to_string()))?;
            if sessions.contains_key(&config.task_id.0) {
                log::warn!(
                    "[opencode] Session already active for task {}",
                    config.task_id.0
                );
                return Err(AgentError::SessionAlreadyActive(config.task_id.0.clone()));
            }
        }
        self.spawn_and_wire(config)
    }

    fn send_follow_up(&self, mut config: AgentRunConfig) -> Result<AgentHandle, AgentError> {
        log::info!(
            "[opencode] Sending follow-up for task {}, prompt_len={}",
            config.task_id.0,
            config.prompt.len()
        );
        {
            let configs = self
                .session_configs
                .lock()
                .map_err(|e| AgentError::Internal(e.to_string()))?;
            if let Some(sc) = configs.get(&config.task_id.0) {
                if let Some(ref m) = sc.model_override {
                    config.model = Some(m.clone());
                }
                if config.project_path.is_none() {
                    config.project_path = sc.config.project_path.clone();
                }
                if config.extra_args.is_empty() {
                    config.extra_args = sc.config.extra_args.clone();
                }
                if config.agent_binary.is_none() {
                    config.agent_binary = sc.config.agent_binary.clone();
                }
            }
        }
        config.is_follow_up = true;
        self.spawn_and_wire(config)
    }

    fn terminate(&self, task_id: &TaskId) -> Result<(), AgentError> {
        log::info!("[opencode] Terminating session {}", task_id.0);
        let mut sessions = self
            .sessions
            .lock()
            .map_err(|e| AgentError::Internal(e.to_string()))?;
        if let Some(mut child) = sessions.remove(&task_id.0) {
            if let Err(e) = child.kill() {
                log::warn!("[opencode] Failed to kill process for {}: {}", task_id.0, e);
            } else {
                log::info!("[opencode] Process killed for {}", task_id.0);
            }
        } else {
            log::info!("[opencode] No active process found for {}", task_id.0);
        }
        if let Ok(mut configs) = self.session_configs.lock() {
            configs.remove(&task_id.0);
        }
        Ok(())
    }

    fn update_model(&self, task_id: &TaskId, model: &str) -> Result<(), AgentError> {
        log::info!(
            "[opencode] Updating model for task {} to '{}'",
            task_id.0,
            model
        );
        let mut configs = self
            .session_configs
            .lock()
            .map_err(|e| AgentError::Internal(e.to_string()))?;
        if let Some(sc) = configs.get_mut(&task_id.0) {
            sc.model_override = if model.is_empty() {
                None
            } else {
                Some(model.to_string())
            };
        }
        Ok(())
    }
}
