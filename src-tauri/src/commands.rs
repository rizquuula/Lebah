use tauri::{AppHandle, Manager, State};
use uuid::Uuid;
use std::process::Command;

use crate::claude::SessionManager;
use crate::db::Database;
use crate::models::{GitStatus, ProjectState, Task, TaskColumn, TaskStatus};

#[tauri::command]
pub fn get_tasks(db: State<'_, Database>) -> Result<Vec<Task>, String> {
    db.get_tasks()
}

#[tauri::command]
pub fn create_task(
    description: String,
    claude_path: Option<String>,
    claude_command: Option<String>,
    db: State<'_, Database>,
) -> Result<Task, String> {
    let id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    db.create_task(&id, &description, &created_at, claude_path.as_deref(), claude_command.as_deref())?;

    Ok(Task {
        id,
        description,
        column: TaskColumn::Todo,
        status: TaskStatus::Idle,
        use_plan: false,
        sort_order: 0,
        created_at,
        claude_path,
        claude_command,
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
    claude_path: Option<String>,
    claude_command: Option<String>,
    db: State<'_, Database>,
    session_manager: State<'_, SessionManager>,
    project_state: State<'_, ProjectState>,
) -> Result<(), String> {
    let project_path = project_state.path.lock().map_err(|e| e.to_string())?.clone();
    db.update_task_status(&id, "Running")?;
    session_manager.run_session(&app, &id, &description, use_plan, claude_path.as_deref(), claude_command.as_deref(), project_path.as_deref())?;

    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_for_thread = Database::new(&app_dir)?;
    let id_clone = id.clone();

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
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

#[tauri::command]
pub fn set_project_path(path: String, project_state: State<'_, ProjectState>) -> Result<(), String> {
    let mut p = project_state.path.lock().map_err(|e| e.to_string())?;
    *p = Some(path);
    Ok(())
}

#[tauri::command]
pub fn get_project_path(project_state: State<'_, ProjectState>) -> Result<Option<String>, String> {
    let p = project_state.path.lock().map_err(|e| e.to_string())?;
    Ok(p.clone())
}

#[tauri::command]
pub fn get_git_status(project_state: State<'_, ProjectState>) -> Result<GitStatus, String> {
    let p = project_state.path.lock().map_err(|e| e.to_string())?;
    let path = p.as_ref().ok_or("No project path set")?;

    // Get branch and tracking info
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    let branch = if branch_output.status.success() {
        String::from_utf8_lossy(&branch_output.stdout).trim().to_string()
    } else {
        return Err("Not a git repository".to_string());
    };

    // Get ahead/behind
    let revlist_output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .current_dir(path)
        .output();

    let (ahead, behind) = if let Ok(output) = revlist_output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let parts: Vec<&str> = text.split_whitespace().collect();
            if parts.len() == 2 {
                (parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0))
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

    // Get changed files count
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to run git status: {}", e))?;

    let changed_files = if status_output.status.success() {
        String::from_utf8_lossy(&status_output.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .count() as u32
    } else {
        0
    };

    Ok(GitStatus {
        branch,
        ahead,
        behind,
        changed_files,
    })
}
