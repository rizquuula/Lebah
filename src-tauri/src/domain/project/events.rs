use crate::domain::project::value_objects::{ProjectId, ProjectPath};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ProjectDomainEvent {
    ProjectSelected {
        project_id: ProjectId,
        path: ProjectPath,
    },
    ProjectConfigUpdated {
        project_id: ProjectId,
    },
}
