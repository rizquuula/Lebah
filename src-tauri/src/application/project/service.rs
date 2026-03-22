use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::application::project::commands::*;
use crate::domain::errors::DomainError;
use crate::domain::project::value_objects::{ProjectConfig, ProjectId};
use crate::domain::repositories::ProjectRepository;

pub struct ProjectApplicationService {
    project_repo: Arc<dyn ProjectRepository>,
    current_project: Arc<std::sync::Mutex<Option<String>>>,
}

impl ProjectApplicationService {
    pub fn new(
        project_repo: Arc<dyn ProjectRepository>,
        current_project: Arc<std::sync::Mutex<Option<String>>>,
    ) -> Self {
        Self {
            project_repo,
            current_project,
        }
    }

    pub fn set_project(&self, cmd: SetProjectCommand) -> Result<(), ApplicationError> {
        let mut config = self.project_repo.load_global_config();
        config.last_project = Some(cmd.path.clone());
        self.project_repo.save_global_config(&config)?;

        *self.current_project.lock()
            .map_err(|e| ApplicationError::Persistence(e.to_string()))? = Some(cmd.path);

        Ok(())
    }

    pub fn get_project(&self) -> Result<Option<String>, ApplicationError> {
        self.current_project
            .lock()
            .map(|g| g.clone())
            .map_err(|e| ApplicationError::Persistence(e.to_string()))
    }

    pub fn get_project_config(&self) -> Result<ProjectConfig, ApplicationError> {
        let path = self.current_project.lock()
            .map_err(|e| ApplicationError::Persistence(e.to_string()))?
            .clone()
            .ok_or(ApplicationError::Domain(DomainError::NoProjectSelected))?;
        let project_id = ProjectId::from_path(&path);
        Ok(self.project_repo.load_project_config(&project_id))
    }

    pub fn set_project_config(&self, cmd: UpdateProjectConfigCommand) -> Result<(), ApplicationError> {
        let path = self.current_project.lock()
            .map_err(|e| ApplicationError::Persistence(e.to_string()))?
            .clone()
            .ok_or(ApplicationError::Domain(DomainError::NoProjectSelected))?;
        let project_id = ProjectId::from_path(&path);
        self.project_repo.save_project_config(&project_id, &cmd.config)?;
        Ok(())
    }
}
