use std::sync::Arc;

use crate::application::git::service::GitApplicationService;
use crate::application::project::service::ProjectApplicationService;
use crate::application::session::service::SessionApplicationService;
use crate::application::task::service::TaskApplicationService;

/// Dependency container passed to Tauri as managed state.
/// Tauri commands receive this as State<'_, AppServices>.
pub struct AppServices {
    pub task_service: Arc<TaskApplicationService>,
    pub session_service: Arc<SessionApplicationService>,
    pub project_service: Arc<ProjectApplicationService>,
    pub git_service: Arc<GitApplicationService>,
}
