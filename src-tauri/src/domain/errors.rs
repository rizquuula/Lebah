use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("No project selected")]
    NoProjectSelected,
    #[error("Invalid column transition from {from} to {to}")]
    InvalidColumnTransition { from: String, to: String },
    #[error("Task is already running")]
    TaskAlreadyRunning,
    #[error("Task is not running")]
    TaskNotRunning,
    #[error("Agent capability not supported: {0}")]
    AgentCapabilityNotSupported(String),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}
