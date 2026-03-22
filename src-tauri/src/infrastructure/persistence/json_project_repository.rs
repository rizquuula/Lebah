use std::sync::Arc;

use crate::domain::errors::DomainError;
use crate::domain::project::value_objects::{GlobalConfig, ProjectConfig, ProjectId};
use crate::domain::repositories::ProjectRepository;
use crate::infrastructure::persistence::path_resolver::PathResolver;

pub struct JsonProjectRepository {
    resolver: Arc<PathResolver>,
}

impl JsonProjectRepository {
    pub fn new(resolver: Arc<PathResolver>) -> Self {
        Self { resolver }
    }
}

impl ProjectRepository for JsonProjectRepository {
    fn load_global_config(&self) -> GlobalConfig {
        let path = self.resolver.global_config_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_global_config(&self, config: &GlobalConfig) -> Result<(), DomainError> {
        let path = self.resolver.global_config_path();
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| DomainError::InvalidValue(e.to_string()))?;
        std::fs::write(path, json).map_err(|e| DomainError::InvalidValue(e.to_string()))
    }

    fn load_project_config(&self, project_id: &ProjectId) -> ProjectConfig {
        let path = self.resolver.project_config_path(project_id);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_project_config(
        &self,
        project_id: &ProjectId,
        config: &ProjectConfig,
    ) -> Result<(), DomainError> {
        let dir = self.resolver.project_dir(project_id);
        std::fs::create_dir_all(&dir).map_err(|e| DomainError::InvalidValue(e.to_string()))?;
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| DomainError::InvalidValue(e.to_string()))?;
        std::fs::write(self.resolver.project_config_path(project_id), json)
            .map_err(|e| DomainError::InvalidValue(e.to_string()))
    }
}
