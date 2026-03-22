use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::application::ports::GitPort;
use crate::domain::git::value_objects::GitStatus;
use crate::domain::project::value_objects::ProjectPath;

pub struct GitApplicationService {
    git_port: Arc<dyn GitPort>,
}

impl GitApplicationService {
    pub fn new(git_port: Arc<dyn GitPort>) -> Self {
        Self { git_port }
    }

    pub fn get_status(&self, project_path: &ProjectPath) -> Result<GitStatus, ApplicationError> {
        self.git_port.get_status(project_path)
    }
}
