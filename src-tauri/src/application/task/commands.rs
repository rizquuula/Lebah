use crate::domain::task::value_objects::{AgentConfig, ExecutionFlags, WorktreeRef};

pub struct CreateTaskCommand {
    pub description: String,
    pub agent_config: AgentConfig,
    pub execution_flags: ExecutionFlags,
    pub worktree: Option<WorktreeRef>,
    pub sort_order: i32,
}

pub struct UpdateTaskCommand {
    pub id: String,
    pub description: String,
    pub column: String,
    pub status: String,
    pub use_plan: bool,
    pub yolo: bool,
    pub sort_order: i32,
    pub agent_path: Option<String>,
    pub agent_command: Option<String>,
    pub model: Option<String>,
}

pub struct DeleteTaskCommand {
    pub id: String,
}

pub struct MoveTaskCommand {
    pub id: String,
    pub column: String,
    pub sort_order: i32,
}

pub struct ResetTaskCommand {
    pub id: String,
}

pub struct MarkTaskStartedCommand {
    pub id: String,
}

pub struct MarkTaskCompletedCommand {
    pub id: String,
    pub success: bool,
}

pub struct MarkTaskStoppedCommand {
    pub id: String,
}
