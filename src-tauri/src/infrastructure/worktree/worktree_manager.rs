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
            log::info!("[worktree] Path does not exist, nothing to remove: {:?}", wt_path);
            return Ok(());
        }

        log::info!("[worktree] Removing worktree: {:?}", wt_path);

        let output = Command::new("git")
            .args(["worktree", "remove", "--force", wt_path.to_str().unwrap_or(worktree.as_str())])
            .current_dir(proj)
            .output();

        let ok = match &output {
            Ok(o) => {
                if !o.status.success() {
                    log::warn!(
                        "[worktree] git worktree remove --force failed: {}",
                        String::from_utf8_lossy(&o.stderr)
                    );
                }
                o.status.success()
            }
            Err(e) => {
                log::error!("[worktree] Failed to run git worktree remove: {}", e);
                false
            }
        };

        if !ok && wt_path.is_dir() {
            log::warn!("[worktree] Falling back to rm -rf: {:?}", wt_path);
            std::fs::remove_dir_all(&wt_path)
                .map_err(|e| {
                    log::error!("[worktree] remove_dir_all failed: {}", e);
                    ApplicationError::Io(e)
                })?;
        }

        // Prune stale worktree refs
        let prune = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(proj)
            .output();

        if let Err(e) = prune {
            log::warn!("[worktree] git worktree prune failed: {}", e);
        }

        log::info!("[worktree] Successfully removed worktree: {}", worktree.as_str());
        Ok(())
    }
}
