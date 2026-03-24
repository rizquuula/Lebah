use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

impl ProjectId {
    pub fn from_path(path: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(path.as_bytes());
        let result = hasher.finalize();
        let hash: String = result[..8].iter().map(|b| format!("{:02x}", b)).collect();
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectPath(pub String);

impl ProjectPath {
    pub fn new(path: String) -> Self {
        Self(path)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProjectPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub review_template: Option<String>,
    pub merge_template: Option<String>,
    pub inprogress_template: Option<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            review_template: Some(
                "Help me to check for test, lint and build error if we not yet do it. Do git commit in the worktree, no need for reading changed files to commit, just use knowledge in session and commit all changes.".to_string(),
            ),
            merge_template: Some(
                "Pull from main branch in the local repository and check for conflict. Fix the conflict gracefully and run build after conflict resolution. Then merge this worktree to the main local branch. Make comprehensive tasks first before executing.".to_string(),
            ),
            inprogress_template: Some(
                "Help me do this task:\n\n <TASK_DESCRIPTION> \n\nMake comprehensive tasks first before executing.".to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub last_project: Option<String>,
}
