use std::process::Command;

use crate::application::errors::ApplicationError;
use crate::application::ports::WorktreePort;
use crate::domain::project::value_objects::ProjectPath;
use crate::domain::task::value_objects::WorktreeRef;

pub struct WorktreeManager;

impl WorktreeManager {
    pub fn new() -> Self {
        Self
    }
}

impl WorktreePort for WorktreeManager {
    fn remove(
        &self,
        project_path: &ProjectPath,
        worktree: &WorktreeRef,
    ) -> Result<(), ApplicationError> {
        let proj = project_path.as_str();
        let wt_path = std::path::Path::new(proj)
            .join(".claude")
            .join("worktrees")
            .join(worktree.as_str());

        if !wt_path.exists() {
            return Ok(());
        }

        let ok = Command::new("git")
            .args(["worktree", "remove", wt_path.to_str().unwrap_or(worktree.as_str())])
            .current_dir(proj)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !ok && wt_path.is_dir() {
            std::fs::remove_dir_all(&wt_path)
                .map_err(|e| ApplicationError::Io(e))?;
        }

        Ok(())
    }
}
