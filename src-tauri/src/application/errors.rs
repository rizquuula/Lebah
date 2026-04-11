use crate::domain::agent::runner::AgentError;
use crate::domain::errors::DomainError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    #[error("Persistence error: {0}")]
    Persistence(String),
    #[error("Git error: {0}")]
    Git(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<ApplicationError> for String {
    fn from(e: ApplicationError) -> Self {
        e.to_string()
    }
}
