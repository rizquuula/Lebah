use tauri::State;

use crate::application::task::commands::*;
use crate::domain::task::value_objects::{AgentConfig, ExecutionFlags, WorktreeRef};
use crate::infrastructure::AppServices;
use crate::presentation::dto::{TaskDto, UpdateTaskInput};

#[tauri::command]
pub fn get_tasks(services: State<'_, AppServices>) -> Result<Vec<TaskDto>, String> {
    services
        .task_service
        .get_tasks()
        .map(|tasks| tasks.into_iter().map(TaskDto::from).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_task(
    description: String,
    claude_path: Option<String>,
    claude_command: Option<String>,
    worktree: Option<String>,
    model: Option<String>,
    services: State<'_, AppServices>,
) -> Result<TaskDto, String> {
    let cmd = CreateTaskCommand {
        description,
        agent_config: AgentConfig {
            agent_name: None,
            agent_path: claude_path,
            agent_command: claude_command,
            model,
        },
        execution_flags: ExecutionFlags::default(),
        worktree: worktree.map(WorktreeRef::new),
        sort_order: 0,
    };
    services
        .task_service
        .create_task(cmd)
        .map(TaskDto::from)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task(
    task: UpdateTaskInput,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    let cmd = UpdateTaskCommand {
        id: task.id,
        description: task.description,
        column: task.column,
        status: task.status,
        use_plan: task.use_plan,
        yolo: task.yolo,
        sort_order: task.sort_order,
        agent_path: task.claude_path,
        agent_command: task.claude_command,
        model: task.model,
    };
    services.task_service.update_task(cmd).map_err(|e| {
        log::error!("[cmd] update_task failed: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub fn delete_task(id: String, services: State<'_, AppServices>) -> Result<(), String> {
    log::info!("[cmd] delete_task id={}", id);

    // Stop any running session first (ignore errors — session may not be active)
    let stop_result = services.session_service.stop_session(
        crate::application::session::commands::StopSessionCommand { task_id: id.clone() },
    );
    if let Err(ref e) = stop_result {
        log::info!("[cmd] No active session to stop for {}: {}", id, e);
    }

    services
        .task_service
        .delete_task(DeleteTaskCommand { id: id.clone() })
        .map_err(|e| {
            log::error!("[cmd] delete_task failed for {}: {}", id, e);
            e.to_string()
        })
}

#[tauri::command]
pub fn move_task(
    id: String,
    column: String,
    sort_order: i32,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    services
        .task_service
        .move_task(MoveTaskCommand { id, column, sort_order })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_task_session(
    id: String,
    services: State<'_, AppServices>,
) -> Result<TaskDto, String> {
    services
        .task_service
        .reset_task(ResetTaskCommand { id: id.clone() })
        .map(TaskDto::from)
        .map_err(|e| {
            log::error!("[cmd] reset_task_session failed for {}: {}", id, e);
            e.to_string()
        })
}
