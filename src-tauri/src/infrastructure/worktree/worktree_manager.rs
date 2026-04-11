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
    fn create(
        &self,
        project_path: &ProjectPath,
        worktree: &WorktreeRef,
    ) -> Result<String, ApplicationError> {
        let proj = project_path.as_str();
        let wt_path = std::path::Path::new(proj)
            .join(".claude")
            .join("worktrees")
            .join(worktree.as_str());

        if wt_path.is_dir() {
            log::info!("[worktree] Already exists, reusing: {:?}", wt_path);
            return Ok(wt_path.to_string_lossy().to_string());
        }

        log::info!("[worktree] Creating worktree: {:?}", wt_path);

        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                wt_path.to_str().unwrap_or(worktree.as_str()),
                "-b",
                worktree.as_str(),
            ])
            .current_dir(proj)
            .output()
            .map_err(|e| {
                log::error!("[worktree] Failed to run git worktree add: {}", e);
                ApplicationError::Io(e)
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If branch already exists, try without -b
            if stderr.contains("already exists") {
                log::info!("[worktree] Branch exists, retrying without -b");
                let output2 = Command::new("git")
                    .args([
                        "worktree",
                        "add",
                        wt_path.to_str().unwrap_or(worktree.as_str()),
                        worktree.as_str(),
                    ])
                    .current_dir(proj)
                    .output()
                    .map_err(ApplicationError::Io)?;

                if !output2.status.success() {
                    let stderr2 = String::from_utf8_lossy(&output2.stderr);
                    log::error!("[worktree] git worktree add failed: {}", stderr2);
                    return Err(ApplicationError::Persistence(format!(
                        "git worktree add failed: {}",
                        stderr2
                    )));
                }
            } else {
                log::error!("[worktree] git worktree add failed: {}", stderr);
                return Err(ApplicationError::Persistence(format!(
                    "git worktree add failed: {}",
                    stderr
                )));
            }
        }

        log::info!(
            "[worktree] Successfully created worktree: {}",
            worktree.as_str()
        );
        Ok(wt_path.to_string_lossy().to_string())
    }

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
            log::info!(
                "[worktree] Path does not exist, nothing to remove: {:?}",
                wt_path
            );
            return Ok(());
        }

        log::info!("[worktree] Removing worktree: {:?}", wt_path);

        let output = Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                wt_path.to_str().unwrap_or(worktree.as_str()),
            ])
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
            std::fs::remove_dir_all(&wt_path).map_err(|e| {
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

        log::info!(
            "[worktree] Successfully removed worktree: {}",
            worktree.as_str()
        );
        Ok(())
    }

    fn apply_links(
        &self,
        project_path: &ProjectPath,
        worktree: &WorktreeRef,
        links: &[String],
    ) -> Result<(), ApplicationError> {
        let proj = std::path::Path::new(project_path.as_str());
        let wt_path = proj
            .join(".claude")
            .join("worktrees")
            .join(worktree.as_str());

        if !wt_path.is_dir() {
            log::warn!(
                "[worktree] apply_links: worktree dir does not exist: {:?}",
                wt_path
            );
            return Ok(());
        }

        for link in links {
            let link = link.trim();
            if link.is_empty() {
                continue;
            }
            let source = proj.join(link);
            if !source.exists() {
                log::debug!(
                    "[worktree] apply_links: source does not exist, skipping: {:?}",
                    source
                );
                continue;
            }
            let dest = wt_path.join(link);

            // Ensure parent directory exists inside worktree
            if let Some(parent) = dest.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(ApplicationError::Io)?;
                }
            }

            if dest.exists() || dest.symlink_metadata().is_ok() {
                log::debug!(
                    "[worktree] apply_links: dest already exists, skipping: {:?}",
                    dest
                );
                continue;
            }

            #[cfg(unix)]
            std::os::unix::fs::symlink(&source, &dest).map_err(|e| {
                log::error!(
                    "[worktree] apply_links: failed to symlink {:?} -> {:?}: {}",
                    dest,
                    source,
                    e
                );
                ApplicationError::Io(e)
            })?;

            #[cfg(windows)]
            {
                if source.is_dir() {
                    std::os::windows::fs::symlink_dir(&source, &dest).map_err(|e| {
                        log::error!(
                            "[worktree] apply_links: failed to symlink dir {:?} -> {:?}: {}",
                            dest,
                            source,
                            e
                        );
                        ApplicationError::Io(e)
                    })?;
                } else {
                    std::os::windows::fs::symlink_file(&source, &dest).map_err(|e| {
                        log::error!(
                            "[worktree] apply_links: failed to symlink file {:?} -> {:?}: {}",
                            dest,
                            source,
                            e
                        );
                        ApplicationError::Io(e)
                    })?;
                }
            }

            log::info!(
                "[worktree] apply_links: symlinked {:?} -> {:?}",
                dest,
                source
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project::value_objects::ProjectPath;
    use crate::domain::task::value_objects::WorktreeRef;
    use std::fs;
    use tempfile::TempDir;

    fn setup_worktree(tmp: &TempDir, wt_name: &str) -> (ProjectPath, WorktreeRef) {
        let wt_dir = tmp.path().join(".claude").join("worktrees").join(wt_name);
        fs::create_dir_all(&wt_dir).unwrap();
        let project_path = ProjectPath::new(tmp.path().to_str().unwrap().to_string());
        let worktree = WorktreeRef::new(wt_name.to_string());
        (project_path, worktree)
    }

    #[test]
    fn apply_links_creates_symlink_for_existing_file() {
        let tmp = TempDir::new().unwrap();
        let (project_path, worktree) = setup_worktree(&tmp, "my-feature");

        // Create a source file in project root
        let source = tmp.path().join(".env");
        fs::write(&source, "SECRET=1").unwrap();

        let mgr = WorktreeManager::new();
        mgr.apply_links(&project_path, &worktree, &[".env".to_string()])
            .unwrap();

        let symlink = tmp
            .path()
            .join(".claude")
            .join("worktrees")
            .join("my-feature")
            .join(".env");
        assert!(symlink.symlink_metadata().is_ok(), "symlink should exist");
        assert!(symlink.is_file(), "symlink should resolve to a file");
    }

    #[test]
    fn apply_links_creates_symlink_for_existing_dir() {
        let tmp = TempDir::new().unwrap();
        let (project_path, worktree) = setup_worktree(&tmp, "my-feature");

        let node_modules = tmp.path().join("node_modules");
        fs::create_dir_all(&node_modules).unwrap();
        fs::write(node_modules.join("pkg.js"), "").unwrap();

        let mgr = WorktreeManager::new();
        mgr.apply_links(&project_path, &worktree, &["node_modules".to_string()])
            .unwrap();

        let symlink = tmp
            .path()
            .join(".claude")
            .join("worktrees")
            .join("my-feature")
            .join("node_modules");
        assert!(symlink.symlink_metadata().is_ok());
        assert!(symlink.is_dir());
    }

    #[test]
    fn apply_links_skips_nonexistent_source() {
        let tmp = TempDir::new().unwrap();
        let (project_path, worktree) = setup_worktree(&tmp, "wt");

        let mgr = WorktreeManager::new();
        // Should not error even though source doesn't exist
        let result = mgr.apply_links(&project_path, &worktree, &["does_not_exist".to_string()]);
        assert!(result.is_ok());

        let dest = tmp
            .path()
            .join(".claude")
            .join("worktrees")
            .join("wt")
            .join("does_not_exist");
        assert!(!dest.exists());
    }

    #[test]
    fn apply_links_skips_already_symlinked_dest() {
        let tmp = TempDir::new().unwrap();
        let (project_path, worktree) = setup_worktree(&tmp, "wt");

        let source = tmp.path().join(".env");
        fs::write(&source, "K=V").unwrap();

        let mgr = WorktreeManager::new();
        mgr.apply_links(&project_path, &worktree, &[".env".to_string()])
            .unwrap();
        // Call again — should not error
        let result = mgr.apply_links(&project_path, &worktree, &[".env".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn apply_links_no_op_when_worktree_missing() {
        let tmp = TempDir::new().unwrap();
        let project_path = ProjectPath::new(tmp.path().to_str().unwrap().to_string());
        let worktree = WorktreeRef::new("nonexistent".to_string());

        let source = tmp.path().join(".env");
        fs::write(&source, "X=1").unwrap();

        let mgr = WorktreeManager::new();
        let result = mgr.apply_links(&project_path, &worktree, &[".env".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn create_reuses_existing_worktree_dir() {
        let tmp = TempDir::new().unwrap();
        let (project_path, worktree) = setup_worktree(&tmp, "existing-wt");

        let mgr = WorktreeManager::new();
        let result = mgr.create(&project_path, &worktree);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.contains("existing-wt"));
        assert!(std::path::Path::new(&path).is_dir());
    }

    #[test]
    fn apply_links_skips_empty_entries() {
        let tmp = TempDir::new().unwrap();
        let (project_path, worktree) = setup_worktree(&tmp, "wt");

        let mgr = WorktreeManager::new();
        let result = mgr.apply_links(
            &project_path,
            &worktree,
            &["".to_string(), "  ".to_string()],
        );
        assert!(result.is_ok());
    }
}
