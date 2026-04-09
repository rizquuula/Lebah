use tauri::State;

use crate::application::project::commands::*;
use crate::domain::git::value_objects::GitStatus;
use crate::domain::project::value_objects::{ProjectConfig, ProjectPath};
use crate::domain::task::value_objects::WorktreeRef;
use crate::infrastructure::AppServices;

#[tauri::command]
pub fn set_project_path(path: String, services: State<'_, AppServices>) -> Result<(), String> {
    services
        .project_service
        .set_project(SetProjectCommand { path })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_path(services: State<'_, AppServices>) -> Result<Option<String>, String> {
    services.project_service.get_project().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent_projects(max_count: usize, services: State<'_, AppServices>) -> Result<Vec<String>, String> {
    services
        .project_service
        .get_recent_projects(GetRecentProjectsCommand { max_count })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_recent_project(path: String, services: State<'_, AppServices>) -> Result<(), String> {
    services
        .project_service
        .remove_recent_project(RemoveRecentProjectCommand { path })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_git_status(services: State<'_, AppServices>) -> Result<GitStatus, String> {
    let path = services
        .project_service
        .get_project()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No project path set".to_string())?;
    services
        .git_service
        .get_status(&ProjectPath::new(path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_config(services: State<'_, AppServices>) -> Result<ProjectConfig, String> {
    services.project_service.get_project_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn git_push(services: State<'_, AppServices>) -> Result<String, String> {
    let path = services
        .project_service
        .get_project()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No project path set".to_string())?;
    services
        .git_service
        .push(&ProjectPath::new(path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_project_config(
    config: ProjectConfig,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    services
        .project_service
        .set_project_config(UpdateProjectConfigCommand { config })
        .map_err(|e| e.to_string())
}

/// Explicitly apply worktree links for a given worktree.
/// Called automatically by run_claude_session, but also exposed so the frontend
/// can trigger a re-link after editing the worktree_links config.
#[tauri::command]
pub fn apply_worktree_links(
    worktree: String,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    let project_path = services
        .project_service
        .get_project()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No project path set".to_string())?;

    let config = services
        .project_service
        .get_project_config()
        .map_err(|e| e.to_string())?;

    let links = config.worktree_links.unwrap_or_default();
    if links.is_empty() {
        return Ok(());
    }

    services
        .worktree_port
        .apply_links(
            &ProjectPath::new(project_path),
            &WorktreeRef::new(worktree),
            &links,
        )
        .map_err(|e| e.to_string())
}
