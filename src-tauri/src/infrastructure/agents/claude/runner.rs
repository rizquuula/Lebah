use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::Child;
use std::sync::{Arc, Mutex};

use crate::domain::agent::runner::{
    AgentCapabilities, AgentError, AgentHandle, AgentRunConfig, AgentRunner,
};
use crate::domain::task::value_objects::TaskId;
use crate::infrastructure::agents::claude::command_builder::ClaudeCommandBuilder;

struct SessionConfig {
    config: AgentRunConfig,
    model_override: Option<String>,
}

pub struct ClaudeRunner {
    sessions: Arc<Mutex<HashMap<String, Child>>>,
    session_configs: Mutex<HashMap<String, SessionConfig>>,
}

impl ClaudeRunner {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_configs: Mutex::new(HashMap::new()),
        }
    }

    fn spawn_and_wire(
        &self,
        config: AgentRunConfig,
    ) -> Result<AgentHandle, AgentError> {
        let mut cmd = ClaudeCommandBuilder::build(&config);

        eprintln!(
            "[claude] Spawning: {} {:?}",
            cmd.get_program().to_string_lossy(),
            cmd.get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        );

        let mut child = cmd.spawn().map_err(|e| {
            AgentError::SpawnFailed(format!("Failed to spawn claude: {}", e))
        })?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let (stdout_tx, stdout_rx) = std::sync::mpsc::channel();
        let (stderr_tx, stderr_rx) = std::sync::mpsc::channel();
        let (exit_tx, exit_rx) = std::sync::mpsc::channel();

        if let Some(stdout) = stdout {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    if stdout_tx.send(line).is_err() {
                        break;
                    }
                }
            });
        }

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

        let task_id_str = config.task_id.0.clone();
        let sessions_arc = Arc::clone(&self.sessions);
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let mut sessions = match sessions_arc.lock() {
                    Ok(s) => s,
                    Err(_) => break,
                };
                if let Some(child) = sessions.get_mut(&task_id_str) {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            sessions.remove(&task_id_str);
                            drop(sessions);
                            let _ = exit_tx.send(status.success());
                            break;
                        }
                        Ok(None) => {}
                        Err(_) => {
                            sessions.remove(&task_id_str);
                            drop(sessions);
                            let _ = exit_tx.send(false);
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });

        {
            let mut sessions = self.sessions.lock()
                .map_err(|e| AgentError::Internal(e.to_string()))?;
            sessions.insert(config.task_id.0.clone(), child);
        }

        {
            let mut configs = self.session_configs.lock()
                .map_err(|e| AgentError::Internal(e.to_string()))?;
            configs.insert(config.task_id.0.clone(), SessionConfig {
                config,
                model_override: None,
            });
        }

        Ok(AgentHandle { stdout_rx, stderr_rx, exit_rx })
    }
}

impl AgentRunner for ClaudeRunner {
    fn name(&self) -> &str {
        "claude"
    }

    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            supports_plan_mode: true,
            supports_session_resume: true,
            supports_worktree: true,
            supports_model_selection: true,
            supports_follow_up: true,
        }
    }

    fn start(&self, config: AgentRunConfig) -> Result<AgentHandle, AgentError> {
        {
            let sessions = self.sessions.lock()
                .map_err(|e| AgentError::Internal(e.to_string()))?;
            if sessions.contains_key(&config.task_id.0) {
                return Err(AgentError::SessionAlreadyActive(config.task_id.0.clone()));
            }
        }
        self.spawn_and_wire(config)
    }

    fn send_follow_up(&self, mut config: AgentRunConfig) -> Result<AgentHandle, AgentError> {
        // Merge model override if set
        {
            let configs = self.session_configs.lock()
                .map_err(|e| AgentError::Internal(e.to_string()))?;
            if let Some(sc) = configs.get(&config.task_id.0) {
                if let Some(ref m) = sc.model_override {
                    config.model = Some(m.clone());
                }
                // Inherit project_path and worktree from saved config if not set
                if config.project_path.is_none() {
                    config.project_path = sc.config.project_path.clone();
                }
                if config.worktree.is_none() {
                    config.worktree = sc.config.worktree.clone();
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
        log::info!("[claude] Terminating session {}", task_id.0);
        let mut sessions = self.sessions.lock()
            .map_err(|e| AgentError::Internal(e.to_string()))?;
        if let Some(mut child) = sessions.remove(&task_id.0) {
            if let Err(e) = child.kill() {
                log::warn!("[claude] Failed to kill process for {}: {}", task_id.0, e);
            } else {
                log::info!("[claude] Process killed for {}", task_id.0);
            }
        } else {
            log::info!("[claude] No active process found for {}", task_id.0);
        }
        // Clean up session config
        if let Ok(mut configs) = self.session_configs.lock() {
            configs.remove(&task_id.0);
        }
        Ok(())
    }

    fn update_model(&self, task_id: &TaskId, model: &str) -> Result<(), AgentError> {
        let mut configs = self.session_configs.lock()
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
