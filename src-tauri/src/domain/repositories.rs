use crate::domain::errors::DomainError;
use crate::domain::project::value_objects::{GlobalConfig, ProjectConfig, ProjectId};
use crate::domain::task::aggregate::Task;
use crate::domain::task::value_objects::TaskId;

pub trait TaskRepository: Send + Sync + 'static {
    fn find_all(&self, project_id: &ProjectId) -> Result<Vec<Task>, DomainError>;
    fn find_by_id(&self, project_id: &ProjectId, task_id: &TaskId) -> Result<Task, DomainError>;
    fn save(&self, project_id: &ProjectId, task: &Task) -> Result<(), DomainError>;
    fn delete(&self, project_id: &ProjectId, task_id: &TaskId) -> Result<(), DomainError>;
}

pub trait ProjectRepository: Send + Sync + 'static {
    fn load_global_config(&self) -> GlobalConfig;
    fn save_global_config(&self, config: &GlobalConfig) -> Result<(), DomainError>;
    fn load_project_config(&self, project_id: &ProjectId) -> ProjectConfig;
    fn save_project_config(
        &self,
        project_id: &ProjectId,
        config: &ProjectConfig,
    ) -> Result<(), DomainError>;
}

pub trait OutputRepository: Send + Sync + 'static {
    fn append(
        &self,
        project_id: &ProjectId,
        task_id: &TaskId,
        line: &str,
    ) -> Result<(), DomainError>;
    fn load_all(&self, project_id: &ProjectId, task_id: &TaskId) -> Vec<String>;
    fn clear(&self, project_id: &ProjectId, task_id: &TaskId) -> Result<(), DomainError>;
}
