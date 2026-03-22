use crate::application::errors::ApplicationError;
use crate::domain::git::value_objects::GitStatus;
use crate::domain::project::value_objects::ProjectPath;
use crate::domain::task::value_objects::{TaskId, WorktreeRef};

pub trait GitPort: Send + Sync + 'static {
    fn get_status(&self, project_path: &ProjectPath) -> Result<GitStatus, ApplicationError>;
}

pub trait WorktreePort: Send + Sync + 'static {
    fn remove(
        &self,
        project_path: &ProjectPath,
        worktree: &WorktreeRef,
    ) -> Result<(), ApplicationError>;
}

pub trait SessionManagerPort: Send + Sync + 'static {
    fn get_live_buffer(&self, task_id: &TaskId) -> Vec<String>;
}
