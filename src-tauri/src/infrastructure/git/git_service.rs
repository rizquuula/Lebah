use std::process::Command;

use crate::application::errors::ApplicationError;
use crate::application::ports::GitPort;
use crate::domain::git::value_objects::GitStatus;
use crate::domain::project::value_objects::ProjectPath;

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
            String::from_utf8_lossy(&branch_output.stdout).trim().to_string()
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

        Ok(GitStatus { branch, ahead, behind, changed_files })
    }
}
