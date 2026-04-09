use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

use crate::infrastructure::terminal::pty_manager::{PtySessionManager, SessionInfo};
use crate::infrastructure::AppServices;

#[tauri::command]
pub fn create_terminal_session(
    app_handle: AppHandle,
    pty: State<'_, Arc<Mutex<PtySessionManager>>>,
    services: State<'_, AppServices>,
    cols: u16,
    rows: u16,
) -> Result<SessionInfo, String> {
    let project_path = services
        .project_service
        .get_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project path set")?;

    let mut manager = pty.lock().map_err(|e| e.to_string())?;
    manager.create_session(&project_path, cols, rows, app_handle)
}

#[tauri::command]
pub fn list_terminal_sessions(
    pty: State<'_, Arc<Mutex<PtySessionManager>>>,
    services: State<'_, AppServices>,
) -> Result<Vec<SessionInfo>, String> {
    let project_path = services
        .project_service
        .get_project()
        .map_err(|e| e.to_string())?;
    let manager = pty.lock().map_err(|e| e.to_string())?;
    Ok(manager.list_sessions(project_path.as_deref()))
}

#[tauri::command]
pub fn write_terminal(
    session_id: String,
    data: String,
    pty: State<'_, Arc<Mutex<PtySessionManager>>>,
) -> Result<(), String> {
    let mut manager = pty.lock().map_err(|e| e.to_string())?;
    manager.write(&session_id, &data)
}

#[tauri::command]
pub fn resize_terminal(
    session_id: String,
    cols: u16,
    rows: u16,
    pty: State<'_, Arc<Mutex<PtySessionManager>>>,
) -> Result<(), String> {
    let manager = pty.lock().map_err(|e| e.to_string())?;
    manager.resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn close_terminal_session(
    session_id: String,
    pty: State<'_, Arc<Mutex<PtySessionManager>>>,
) -> Result<(), String> {
    let mut manager = pty.lock().map_err(|e| e.to_string())?;
    manager.close_session(&session_id);
    Ok(())
}
