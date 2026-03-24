use crate::application::errors::ApplicationError;
use crate::domain::git::value_objects::GitStatus;
use crate::domain::project::value_objects::ProjectPath;
use crate::domain::task::value_objects::{TaskId, WorktreeRef};

pub trait GitPort: Send + Sync + 'static {
    fn get_status(&self, project_path: &ProjectPath) -> Result<GitStatus, ApplicationError>;
    /// Returns (lines_added, lines_removed) for a worktree branch vs main.
    fn get_diff_stat(
        &self,
        project_path: &ProjectPath,
        worktree: &WorktreeRef,
    ) -> Result<(i32, i32), ApplicationError>;
    /// Push the current branch to its upstream remote.
    fn push(&self, project_path: &ProjectPath) -> Result<String, ApplicationError>;
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
