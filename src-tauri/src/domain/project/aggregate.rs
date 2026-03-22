use crate::domain::project::events::ProjectDomainEvent;
use crate::domain::project::value_objects::{ProjectConfig, ProjectId, ProjectPath};

#[allow(dead_code)]
pub struct Project {
    id: ProjectId,
    path: ProjectPath,
    config: ProjectConfig,
}

#[allow(dead_code)]
impl Project {
    pub fn open(path: ProjectPath) -> (Project, ProjectDomainEvent) {
        let id = ProjectId::from_path(path.as_str());
        let event = ProjectDomainEvent::ProjectSelected {
            project_id: id.clone(),
            path: path.clone(),
        };
        (Project { id, path, config: ProjectConfig::default() }, event)
    }

    pub fn with_config(mut self, config: ProjectConfig) -> Self {
        self.config = config;
        self
    }

    pub fn update_config(&mut self, config: ProjectConfig) -> ProjectDomainEvent {
        self.config = config;
        ProjectDomainEvent::ProjectConfigUpdated {
            project_id: self.id.clone(),
        }
    }

    pub fn id(&self) -> &ProjectId { &self.id }
    pub fn path(&self) -> &ProjectPath { &self.path }
    pub fn config(&self) -> &ProjectConfig { &self.config }
}
