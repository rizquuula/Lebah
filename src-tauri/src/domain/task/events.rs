use crate::domain::task::value_objects::{TaskId, TaskColumn, WorktreeRef};
use crate::domain::project::value_objects::ProjectId;

#[allow(dead_code, clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum TaskDomainEvent {
    TaskCreated {
        task_id: TaskId,
        project_id: ProjectId,
    },
    TaskMoved {
        task_id: TaskId,
        from: TaskColumn,
        to: TaskColumn,
    },
    TaskDeleted {
        task_id: TaskId,
        project_id: ProjectId,
        worktree: Option<WorktreeRef>,
    },
    TaskReset {
        old_task_id: TaskId,
        new_task_id: TaskId,
        project_id: ProjectId,
    },
    TaskStarted {
        task_id: TaskId,
    },
    TaskCompleted {
        task_id: TaskId,
        success: bool,
    },
    TaskStopped {
        task_id: TaskId,
    },
}
