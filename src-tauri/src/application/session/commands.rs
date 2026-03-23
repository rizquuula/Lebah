use crate::domain::agent::runner::PermissionMode;
use crate::domain::project::value_objects::ProjectPath;
use crate::domain::task::value_objects::WorktreeRef;

pub struct StartSessionCommand {
    pub task_id: String,
    pub description: String,
    pub permission_mode: PermissionMode,
    pub agent_path: Option<String>,
    pub worktree: Option<WorktreeRef>,
    pub project_path: Option<ProjectPath>,
    pub model: Option<String>,
    pub agent_name: Option<String>,
}

pub struct StopSessionCommand {
    pub task_id: String,
}

pub struct SendInputCommand {
    pub task_id: String,
    pub input: String,
    pub model: Option<String>,
    pub yolo: bool,
}
