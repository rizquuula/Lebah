use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

use crate::infrastructure::terminal::pty_manager::PtyManager;
use crate::infrastructure::AppServices;

#[tauri::command]
pub fn spawn_terminal(
    app_handle: AppHandle,
    pty: State<'_, Arc<Mutex<PtyManager>>>,
    services: State<'_, AppServices>,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    let project_path = services
        .project_service
        .get_project()
        .map_err(|e| e.to_string())?
        .ok_or("No project path set")?;

    let mut manager = pty.lock().map_err(|e| e.to_string())?;
    manager.spawn(&project_path, cols, rows, app_handle)
}

#[tauri::command]
pub fn write_terminal(
    data: String,
    pty: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<(), String> {
    let mut manager = pty.lock().map_err(|e| e.to_string())?;
    manager.write(&data)
}

#[tauri::command]
pub fn resize_terminal(
    cols: u16,
    rows: u16,
    pty: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<(), String> {
    let manager = pty.lock().map_err(|e| e.to_string())?;
    manager.resize(cols, rows)
}

#[tauri::command]
pub fn close_terminal(
    pty: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<(), String> {
    let mut manager = pty.lock().map_err(|e| e.to_string())?;
    manager.kill();
    Ok(())
}
