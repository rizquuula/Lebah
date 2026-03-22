use tauri::State;

use crate::application::project::commands::*;
use crate::domain::git::value_objects::GitStatus;
use crate::domain::project::value_objects::{ProjectConfig, ProjectPath};
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
pub fn set_project_config(
    config: ProjectConfig,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    services
        .project_service
        .set_project_config(UpdateProjectConfigCommand { config })
        .map_err(|e| e.to_string())
}
