use tauri::{AppHandle, State};

use crate::application::session::commands::*;
use crate::domain::agent::runner::PermissionMode;
use crate::domain::project::value_objects::ProjectPath;
use crate::domain::task::value_objects::WorktreeRef;
use crate::infrastructure::AppServices;

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn run_claude_session(
    _app: AppHandle,
    id: String,
    description: String,
    use_plan: bool,
    yolo: bool,
    claude_path: Option<String>,
    claude_command: Option<String>,
    worktree: Option<String>,
    model: Option<String>,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    let project_path = services.project_service.get_project().map_err(|e| e.to_string())?;

    let permission_mode = if use_plan {
        PermissionMode::Plan
    } else if yolo {
        PermissionMode::Sandbox
    } else {
        PermissionMode::Full
    };

    let cmd = StartSessionCommand {
        task_id: id.clone(),
        description,
        permission_mode,
        agent_path: claude_path,
        agent_command: claude_command,
        worktree: worktree.map(WorktreeRef::new),
        project_path: project_path.map(ProjectPath::new),
        model,
        agent_name: None,
    };

    services.session_service.start_session(cmd).map_err(|e| {
        log::error!("[cmd] run_claude_session failed for {}: {}", id, e);
        e.to_string()
    })
}

#[tauri::command]
pub fn stop_claude_session(
    id: String,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    services
        .session_service
        .stop_session(StopSessionCommand { task_id: id.clone() })
        .map_err(|e| {
            log::error!("[cmd] stop_claude_session failed for {}: {}", id, e);
            e.to_string()
        })
}

#[tauri::command]
pub fn send_input(
    _app: AppHandle,
    id: String,
    input: String,
    model: Option<String>,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    services
        .session_service
        .send_input(SendInputCommand {
            task_id: id,
            input,
            model,
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_output_buffer(
    id: String,
    services: State<'_, AppServices>,
) -> Result<Vec<String>, String> {
    Ok(services.session_service.get_output_buffer(&id))
}
