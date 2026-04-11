use std::process::Command;

use crate::application::errors::ApplicationError;
use crate::application::ports::GitPort;
use crate::domain::git::value_objects::GitStatus;
use crate::domain::project::value_objects::ProjectPath;
use crate::domain::task::value_objects::WorktreeRef;

pub struct GitInfraService;

impl GitInfraService {
    pub fn new() -> Self {
        Self
    }
}

impl GitPort for GitInfraService {
    fn get_status(&self, project_path: &ProjectPath) -> Result<GitStatus, ApplicationError> {
        let path = project_path.as_str();

        let branch_output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(path)
            .output()
            .map_err(|e| ApplicationError::Git(format!("Failed to run git: {}", e)))?;

        let branch = if branch_output.status.success() {
            String::from_utf8_lossy(&branch_output.stdout)
                .trim()
                .to_string()
        } else {
            return Err(ApplicationError::Git("Not a git repository".to_string()));
        };

        let revlist_output = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .current_dir(path)
            .output();

        let (ahead, behind) = if let Ok(output) = revlist_output {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() == 2 {
                    (parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0))
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        };

        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(path)
            .output()
            .map_err(|e| ApplicationError::Git(format!("Failed to run git status: {}", e)))?;

        let changed_files = if status_output.status.success() {
            String::from_utf8_lossy(&status_output.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .count() as u32
        } else {
            0
        };

        Ok(GitStatus {
            branch,
            ahead,
            behind,
            changed_files,
        })
    }

    fn get_diff_stat(
        &self,
        project_path: &ProjectPath,
        worktree: &WorktreeRef,
    ) -> Result<(i32, i32), ApplicationError> {
        let wt_path = std::path::Path::new(project_path.as_str())
            .join(".claude")
            .join("worktrees")
            .join(worktree.as_str());

        let wt_dir = if wt_path.is_dir() {
            wt_path.to_str().unwrap_or("").to_string()
        } else {
            // Fallback to project root if worktree path doesn't exist
            project_path.as_str().to_string()
        };

        // Get the main branch name
        let main_branch = Command::new("git")
            .args(["rev-parse", "--verify", "--quiet", "main"])
            .current_dir(&wt_dir)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some("main")
                } else {
                    None
                }
            })
            .unwrap_or("master");

        let output = Command::new("git")
            .args(["diff", "--numstat", &format!("{}...HEAD", main_branch)])
            .current_dir(&wt_dir)
            .output()
            .map_err(|e| ApplicationError::Git(format!("Failed to run git diff: {}", e)))?;

        if !output.status.success() {
            return Err(ApplicationError::Git(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let (mut added, mut removed) = (0i32, 0i32);
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                added += parts[0].parse::<i32>().unwrap_or(0);
                removed += parts[1].parse::<i32>().unwrap_or(0);
            }
        }

        Ok((added, removed))
    }

    fn push(&self, project_path: &ProjectPath) -> Result<String, ApplicationError> {
        let path = project_path.as_str();

        // First, fetch to ensure we have the latest remote state
        let fetch_output = Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(path)
            .output()
            .map_err(|e| ApplicationError::Git(format!("Failed to run git fetch: {}", e)))?;

        if !fetch_output.status.success() {
            let stderr = String::from_utf8_lossy(&fetch_output.stderr).to_string();
            return Err(ApplicationError::Git(format!(
                "Git fetch failed: {}",
                stderr.trim()
            )));
        }

        // Then push current branch to origin
        let push_output = Command::new("git")
            .args(["push", "-u", "origin", "HEAD"])
            .current_dir(path)
            .output()
            .map_err(|e| ApplicationError::Git(format!("Failed to run git push: {}", e)))?;

        if !push_output.status.success() {
            let stderr = String::from_utf8_lossy(&push_output.stderr).to_string();
            return Err(ApplicationError::Git(format!(
                "Git push failed: {}",
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&push_output.stdout).to_string();
        Ok(stdout.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> GitInfraService {
        GitInfraService::new()
    }

    #[test]
    fn push_fails_on_non_git_directory() {
        let tmp = std::env::temp_dir();
        let path = ProjectPath::new(tmp.to_string_lossy().to_string());
        let result = service().push(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("fetch failed") || err_msg.contains("Failed"),
            "unexpected error: {}",
            err_msg
        );
    }

    #[test]
    fn push_fails_on_nonexistent_path() {
        let path = ProjectPath::new("/tmp/nonexistent-lebah-test-dir".to_string());
        let result = service().push(&path);
        assert!(result.is_err());
    }

    #[test]
    fn get_status_fails_on_non_git_directory() {
        let tmp = std::env::temp_dir();
        let path = ProjectPath::new(tmp.to_string_lossy().to_string());
        let result = service().get_status(&path);
        assert!(result.is_err());
    }

    #[test]
    fn push_fails_on_repo_without_remote() {
        let tmp = tempfile::tempdir().expect("create temp dir");
        let dir = tmp.path();

        // Init a git repo with no remote
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .expect("git init");
        std::process::Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(dir)
            .output()
            .expect("git commit");

        let path = ProjectPath::new(dir.to_string_lossy().to_string());
        let result = service().push(&path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("fetch failed") || err_msg.contains("push failed"),
            "unexpected error: {}",
            err_msg
        );
    }
}
