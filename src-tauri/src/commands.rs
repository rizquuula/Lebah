use std::process::Command;
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::claude::SessionManager;
use crate::models::{GitStatus, Task, TaskColumn, TaskStatus};
use crate::storage::Storage;

#[tauri::command]
pub fn get_tasks(storage: State<'_, Storage>) -> Result<Vec<Task>, String> {
    storage.get_tasks()
}

#[tauri::command]
pub fn create_task(
    description: String,
    claude_path: Option<String>,
    claude_command: Option<String>,
    worktree: Option<String>,
    storage: State<'_, Storage>,
) -> Result<Task, String> {
    let id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    storage.create_task(
        &id,
        &description,
        &created_at,
        claude_path.as_deref(),
        claude_command.as_deref(),
        worktree.as_deref(),
    )?;

    Ok(Task {
        id,
        description,
        column: TaskColumn::Todo,
        status: TaskStatus::Idle,
        use_plan: false,
        yolo: true,
        sort_order: 0,
        created_at,
        claude_path,
        claude_command,
        worktree,
        has_run: false,
    })
}

#[tauri::command]
pub fn update_task(task: Task, storage: State<'_, Storage>) -> Result<(), String> {
    storage.update_task(&task)
}

#[tauri::command]
pub fn check_path_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

#[tauri::command]
pub fn delete_task(id: String, storage: State<'_, Storage>) -> Result<(), String> {
    if let Ok(Some(worktree)) = storage.get_task_worktree(&id) {
        let path = std::path::Path::new(&worktree);
        if path.is_dir() {
            std::fs::remove_dir_all(path)
                .map_err(|e| format!("Failed to delete worktree: {}", e))?;
        }
    }
    let _ = storage.clear_output_lines(&id);
    storage.delete_task(&id)
}

#[tauri::command]
pub fn move_task(
    id: String,
    column: String,
    sort_order: i32,
    storage: State<'_, Storage>,
) -> Result<(), String> {
    TaskColumn::from_str(&column)?;
    storage.move_task(&id, &column, sort_order)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn run_claude_session(
    app: AppHandle,
    id: String,
    description: String,
    use_plan: bool,
    yolo: bool,
    claude_path: Option<String>,
    claude_command: Option<String>,
    worktree: Option<String>,
    storage: State<'_, Storage>,
    session_manager: State<'_, SessionManager>,
) -> Result<(), String> {
    let project_path = storage.get_project()?;
    eprintln!("[session] run_claude_session: task={} use_plan={} yolo={} project={:?} claude_path={:?} claude_command={:?} worktree={:?}",
        id, use_plan, yolo, project_path, claude_path, claude_command, worktree);
    storage.update_task_status(&id, "Running")?;
    let _ = storage.clear_output_lines(&id);

    if let Err(e) = session_manager.run_session(
        &app,
        &id,
        &description,
        use_plan,
        yolo,
        claude_path.as_deref(),
        claude_command.as_deref(),
        worktree.as_deref(),
        project_path.as_deref(),
    ) {
        eprintln!("[session] Failed to start session for task={}: {}", id, e);
        let _ = storage.update_task_status(&id, "Failed");
        return Err(e);
    }
    let _ = storage.set_task_has_run(&id, true);
    eprintln!("[session] Session started successfully for task={}", id);

    // Watch for process exit and update status
    let pp = project_path.unwrap_or_default();
    let id_clone = id.clone();
    let sessions_arc = session_manager.sessions_arc();
    let thread_storage = Storage::new().expect("Failed to init storage for thread");

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let mut sessions = match sessions_arc.lock() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[monitor] task={} mutex poisoned: {}", id_clone, e);
                    break;
                }
            };
            if let Some(child) = sessions.get_mut(&id_clone) {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let final_status = if status.success() { "Success" } else { "Failed" };
                        eprintln!("[monitor] task={} exited: code={:?} -> {}", id_clone, status.code(), final_status);
                        sessions.remove(&id_clone);
                        drop(sessions);
                        let _ = thread_storage.update_task_status_for(&pp, &id_clone, final_status);
                        break;
                    }
                    Ok(None) => {
                        eprintln!("[monitor] task={} still running", id_clone);
                    }
                    Err(e) => {
                        eprintln!("[monitor] task={} try_wait error: {}", id_clone, e);
                        sessions.remove(&id_clone);
                        drop(sessions);
                        let _ = thread_storage.update_task_status_for(&pp, &id_clone, "Failed");
                        break;
                    }
                }
            } else {
                eprintln!("[monitor] task={} not found in sessions, exiting monitor", id_clone);
                break;
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn get_output_buffer(
    id: String,
    session_manager: State<'_, SessionManager>,
    storage: State<'_, Storage>,
) -> Result<Vec<String>, String> {
    let buf = session_manager.get_output_buffer(&id)?;
    if !buf.is_empty() {
        return Ok(buf);
    }
    Ok(storage.load_output_lines(&id))
}

#[tauri::command]
pub fn send_input(
    id: String,
    input: String,
    session_manager: State<'_, SessionManager>,
) -> Result<(), String> {
    session_manager.send_input(&id, &input)
}

#[tauri::command]
pub fn stop_claude_session(
    id: String,
    storage: State<'_, Storage>,
    session_manager: State<'_, SessionManager>,
) -> Result<(), String> {
    eprintln!("[session] stop_claude_session: task={}", id);
    session_manager.stop_session(&id)?;
    storage.update_task_status(&id, "Idle")?;
    eprintln!("[session] Session stopped for task={}", id);
    Ok(())
}

#[tauri::command]
pub fn reset_task_session(
    id: String,
    storage: State<'_, Storage>,
) -> Result<Task, String> {
    let old_task = storage.get_task(&id)?;

    // Remove worktree directory if it exists
    if let Some(ref wt) = old_task.worktree {
        let path = std::path::Path::new(wt);
        if path.is_dir() {
            std::fs::remove_dir_all(path)
                .map_err(|e| format!("Failed to remove worktree: {}", e))?;
        }
    }

    // Delete old task from storage
    let _ = storage.clear_output_lines(&id);
    storage.delete_task(&id)?;

    // Create replacement with new UUID and same settings
    let new_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    storage.create_task(
        &new_id,
        &old_task.description,
        &created_at,
        old_task.claude_path.as_deref(),
        old_task.claude_command.as_deref(),
        old_task.worktree.as_deref(),
    )?;
    storage.move_task(&new_id, old_task.column.to_str(), old_task.sort_order)?;
    storage.set_task_settings(&new_id, old_task.use_plan, old_task.yolo)?;

    storage.get_task(&new_id)
}

#[tauri::command]
pub fn set_project_path(path: String, storage: State<'_, Storage>) -> Result<(), String> {
    storage.set_project(&path)
}

#[tauri::command]
pub fn get_project_path(storage: State<'_, Storage>) -> Result<Option<String>, String> {
    storage.get_project()
}

#[tauri::command]
pub fn get_git_status(storage: State<'_, Storage>) -> Result<GitStatus, String> {
    let project = storage.get_project()?;
    let path = project.as_ref().ok_or("No project path set")?;

    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    let branch = if branch_output.status.success() {
        String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string()
    } else {
        return Err("Not a git repository".to_string());
    };

    let revlist_output = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .current_dir(path)
        .output();

    let (ahead, behind) = if let Ok(output) = revlist_output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let parts: Vec<&str> = text.split_whitespace().collect();
            if parts.len() == 2 {
                (
                    parts[0].parse().unwrap_or(0),
                    parts[1].parse().unwrap_or(0),
                )
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        }
    } else {
        (0, 0)
    };

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
