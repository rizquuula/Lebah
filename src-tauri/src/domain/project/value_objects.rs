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
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            review_template: Some(
                "Help me to check for test, lint and build error if you not yet do it. Then do commit in the worktree.".to_string(),
            ),
            merge_template: Some(
                "Pull from main branch and check for conflict. Fix the conflict. Then merge this worktree to main.".to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub last_project: Option<String>,
}
