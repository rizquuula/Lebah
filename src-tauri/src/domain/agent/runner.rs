use std::collections::HashMap;
use std::sync::mpsc;
use thiserror::Error;

use crate::domain::project::value_objects::ProjectPath;
use crate::domain::task::value_objects::{TaskId, WorktreeRef};

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Failed to spawn agent process: {0}")]
    SpawnFailed(String),
    #[error("Capability not supported by agent '{agent}': {capability}")]
    NotSupported { agent: String, capability: String },
    #[error("Session already active for task: {0}")]
    SessionAlreadyActive(String),
    #[error("No session config found for task: {0}")]
    NoSessionConfig(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// What optional features an agent implementation supports.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AgentCapabilities {
    pub supports_plan_mode: bool,
    pub supports_session_resume: bool,
    pub supports_worktree: bool,
    pub supports_model_selection: bool,
    pub supports_follow_up: bool,
}

/// Permission/sandbox level for agent execution.
#[derive(Debug, Clone)]
pub enum PermissionMode {
    /// Restricted: agent can only read/plan
    Plan,
    /// Full access
    Full,
    /// Skip all permission checks (sandbox)
    Sandbox,
}

/// Complete configuration for starting or resuming an agent session.
#[derive(Debug, Clone)]
pub struct AgentRunConfig {
    pub task_id: TaskId,
    pub prompt: String,
    pub project_path: Option<ProjectPath>,
    pub worktree: Option<WorktreeRef>,
    pub model: Option<String>,
    pub permission_mode: PermissionMode,
    /// Extra CLI arguments passed through verbatim
    pub extra_args: Vec<String>,
    /// Whether this is a follow-up to an existing session
    pub is_follow_up: bool,
    /// Path override for the agent binary
    pub agent_binary: Option<String>,
    /// Environment variables to pass to the agent process
    pub env_vars: HashMap<String, String>,
}

/// Handles for interacting with a running agent process.
pub struct AgentHandle {
    pub stdout_rx: mpsc::Receiver<String>,
    pub stderr_rx: mpsc::Receiver<String>,
    pub exit_rx: mpsc::Receiver<bool>, // true = success
}

/// The trait every AI coding agent backend must implement.
/// Implement this trait to add support for a new agent (Codex, Aider, etc.)
pub trait AgentRunner: Send + Sync + 'static {
    /// Human-readable agent name, e.g. "claude", "codex"
    fn name(&self) -> &str;

    /// What this agent supports — used for runtime capability checks and future agent selection UI
    #[allow(dead_code)]
    fn capabilities(&self) -> AgentCapabilities;

    /// Start a new session or resume an existing one.
    fn start(&self, config: AgentRunConfig) -> Result<AgentHandle, AgentError>;

    /// Send a follow-up message to a running/completed session.
    fn send_follow_up(&self, config: AgentRunConfig) -> Result<AgentHandle, AgentError>;

    /// Forcibly terminate a session.
    fn terminate(&self, task_id: &TaskId) -> Result<(), AgentError>;

    /// Update the model for future follow-up messages.
    fn update_model(&self, task_id: &TaskId, model: &str) -> Result<(), AgentError>;
}
