use std::collections::HashMap;

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
    #[serde(default)]
    pub claude_path: Option<String>,
    #[serde(default)]
    pub worktree_model: Option<String>,
    #[serde(default)]
    pub default_use_plan: Option<bool>,
    #[serde(default)]
    pub default_yolo: Option<bool>,
    #[serde(default)]
    pub default_auto: Option<bool>,
    #[serde(default)]
    pub env_vars: Option<HashMap<String, String>>,
    #[serde(default)]
    pub disabled_env_var_keys: Option<Vec<String>>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("IS_SANDBOX".to_string(), "0".to_string());
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
            claude_path: None,
            worktree_model: Some("haiku".to_string()),
            default_use_plan: Some(false),
            default_yolo: Some(true),
            default_auto: Some(false),
            env_vars: Some(env_vars),
            disabled_env_var_keys: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub last_project: Option<String>,
    #[serde(default)]
    pub recent_projects: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_id_is_deterministic() {
        let a = ProjectId::from_path("/some/path");
        let b = ProjectId::from_path("/some/path");
        assert_eq!(a, b);
    }

    #[test]
    fn project_id_differs_for_different_paths() {
        let a = ProjectId::from_path("/path/a");
        let b = ProjectId::from_path("/path/b");
        assert_ne!(a, b);
    }

    #[test]
    fn project_id_is_16_hex_chars() {
        let id = ProjectId::from_path("/any/path");
        assert_eq!(id.as_str().len(), 16);
        assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn project_config_default_has_env_var() {
        let cfg = ProjectConfig::default();
        let vars = cfg.env_vars.as_ref().unwrap();
        assert_eq!(vars.get("IS_SANDBOX").map(|s| s.as_str()), Some("0"));
    }

    #[test]
    fn project_config_default_has_templates() {
        let cfg = ProjectConfig::default();
        assert!(cfg.review_template.is_some());
        assert!(cfg.merge_template.is_some());
        assert!(cfg.inprogress_template.is_some());
    }
}
