use serde::{Deserialize, Serialize};

use crate::domain::task::aggregate::Task;

/// Frontend-facing Task DTO — serde-annotated, decoupled from domain aggregate.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskDto {
    pub id: String,
    pub description: String,
    pub column: String,
    pub status: String,
    pub use_plan: bool,
    pub yolo: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub claude_path: Option<String>,
    pub worktree: Option<String>,
    pub has_run: bool,
    pub model: Option<String>,
}

impl From<Task> for TaskDto {
    fn from(t: Task) -> Self {
        Self {
            id: t.id().to_string(),
            description: t.description().to_string(),
            column: t.column().as_str().to_string(),
            status: t.status().as_str().to_string(),
            use_plan: t.execution_flags().use_plan,
            yolo: t.execution_flags().yolo,
            sort_order: t.sort_order(),
            created_at: t.created_at().to_rfc3339(),
            completed_at: t.completed_at().map(|dt| dt.to_rfc3339()),
            claude_path: t.agent_config().agent_path.clone(),
            worktree: t.worktree().map(|w| w.0.clone()),
            has_run: t.has_run(),
            model: t.agent_config().model.clone(),
        }
    }
}

/// Frontend-facing input for updating a task
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    pub id: String,
    pub description: String,
    pub column: String,
    pub status: String,
    pub use_plan: bool,
    pub yolo: bool,
    pub sort_order: i32,
    pub claude_path: Option<String>,
    pub worktree: Option<String>,
    pub has_run: bool,
    pub model: Option<String>,
}

