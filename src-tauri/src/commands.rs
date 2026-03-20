use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::claude::SessionManager;
use crate::db::Database;
use crate::models::{Task, TaskColumn, TaskStatus};

#[tauri::command]
pub fn get_tasks(db: State<'_, Database>) -> Result<Vec<Task>, String> {
    db.get_tasks()
}

#[tauri::command]
pub fn create_task(description: String, db: State<'_, Database>) -> Result<Task, String> {
    let id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    db.create_task(&id, &description, &created_at)?;

    Ok(Task {
        id,
        description,
        column: TaskColumn::Todo,
        status: TaskStatus::Idle,
        use_plan: false,
        sort_order: 0,
        created_at,
    })
}

#[tauri::command]
pub fn update_task(task: Task, db: State<'_, Database>) -> Result<(), String> {
    db.update_task(&task)
}

#[tauri::command]
pub fn delete_task(id: String, db: State<'_, Database>) -> Result<(), String> {
    db.delete_task(&id)
}

#[tauri::command]
pub fn move_task(id: String, column: String, sort_order: i32, db: State<'_, Database>) -> Result<(), String> {
    TaskColumn::from_str(&column)?;
    db.move_task(&id, &column, sort_order)
}

#[tauri::command]
pub fn run_claude_session(
    app: AppHandle,
    id: String,
    description: String,
    use_plan: bool,
    db: State<'_, Database>,
    session_manager: State<'_, SessionManager>,
) -> Result<(), String> {
    db.update_task_status(&id, "Running")?;
    session_manager.run_session(&app, &id, &description, use_plan)?;

    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_for_thread = Database::new(&app_dir)?;
    let id_clone = id.clone();

    std::thread::spawn(move || {
        // Simple polling to detect when session ends
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            // For now, just break — real monitoring would check child exit
            break;
        }
        let _ = db_for_thread.update_task_status(&id_clone, "Success");
    });

    Ok(())
}

#[tauri::command]
pub fn stop_claude_session(
    id: String,
    db: State<'_, Database>,
    session_manager: State<'_, SessionManager>,
) -> Result<(), String> {
    session_manager.stop_session(&id)?;
    db.update_task_status(&id, "Idle")?;
    Ok(())
}
