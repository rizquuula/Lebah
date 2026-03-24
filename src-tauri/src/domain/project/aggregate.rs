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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project::value_objects::{ProjectConfig, ProjectPath};

    #[test]
    fn open_derives_id_from_path() {
        let path = ProjectPath::new("/my/project".to_string());
        let (project, _event) = Project::open(path.clone());
        let expected_id = crate::domain::project::value_objects::ProjectId::from_path(path.as_str());
        assert_eq!(project.id(), &expected_id);
        assert_eq!(project.path().as_str(), "/my/project");
    }

    #[test]
    fn with_config_replaces_config() {
        let (project, _) = Project::open(ProjectPath::new("/p".to_string()));
        let mut cfg = ProjectConfig::default();
        cfg.claude_path = Some("/custom/claude".to_string());
        let project = project.with_config(cfg);
        assert_eq!(project.config().claude_path.as_deref(), Some("/custom/claude"));
    }

    #[test]
    fn update_config_mutates_in_place() {
        let (mut project, _) = Project::open(ProjectPath::new("/p".to_string()));
        let mut cfg = ProjectConfig::default();
        cfg.worktree_model = Some("opus".to_string());
        project.update_config(cfg);
        assert_eq!(project.config().worktree_model.as_deref(), Some("opus"));
    }
}
