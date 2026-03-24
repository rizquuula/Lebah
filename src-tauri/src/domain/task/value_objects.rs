use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskColumn {
    Todo,
    InProgress,
    Review,
    Merge,
    Completed,
}

impl TaskColumn {
    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "Todo" => Ok(TaskColumn::Todo),
            "InProgress" => Ok(TaskColumn::InProgress),
            "Review" => Ok(TaskColumn::Review),
            "Merge" => Ok(TaskColumn::Merge),
            "Completed" => Ok(TaskColumn::Completed),
            _ => Err(DomainError::InvalidValue(format!("Unknown column: {}", s))),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TaskColumn::Todo => "Todo",
            TaskColumn::InProgress => "InProgress",
            TaskColumn::Review => "Review",
            TaskColumn::Merge => "Merge",
            TaskColumn::Completed => "Completed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Idle,
    Running,
    Success,
    Failed,
    Warning,
    Waiting,
    Canceled,
}

impl TaskStatus {
    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "Idle" => Ok(TaskStatus::Idle),
            "Running" => Ok(TaskStatus::Running),
            "Success" => Ok(TaskStatus::Success),
            "Failed" => Ok(TaskStatus::Failed),
            "Warning" => Ok(TaskStatus::Warning),
            "Waiting" => Ok(TaskStatus::Waiting),
            "Canceled" => Ok(TaskStatus::Canceled),
            _ => Err(DomainError::InvalidValue(format!("Unknown status: {}", s))),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Idle => "Idle",
            TaskStatus::Running => "Running",
            TaskStatus::Success => "Success",
            TaskStatus::Failed => "Failed",
            TaskStatus::Warning => "Warning",
            TaskStatus::Waiting => "Waiting",
            TaskStatus::Canceled => "Canceled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorktreeRef(pub String);

impl WorktreeRef {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Configuration for the AI agent to use for this task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Name of the registered agent to use (None = use default)
    pub agent_name: Option<String>,
    /// Path to the agent binary (None = use PATH)
    pub agent_path: Option<String>,
    /// Model override
    pub model: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_name: None,
            agent_path: None,
            model: None,
        }
    }
}

/// Execution permission flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFlags {
    /// Use plan mode (restricted permissions)
    pub use_plan: bool,
    /// Skip permission checks (yolo/sandbox mode)
    pub yolo: bool,
    /// Auto-advance through columns on success
    pub auto: bool,
}

impl Default for ExecutionFlags {
    fn default() -> Self {
        Self {
            use_plan: false,
            yolo: true,
            auto: false,
        }
    }
}
