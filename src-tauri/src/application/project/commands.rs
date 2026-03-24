use crate::domain::project::value_objects::ProjectConfig;

pub struct SetProjectCommand {
    pub path: String,
}

pub struct UpdateProjectConfigCommand {
    pub config: ProjectConfig,
}

pub struct GetRecentProjectsCommand {
    pub max_count: usize,
}
