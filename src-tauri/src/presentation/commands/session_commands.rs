use std::collections::HashMap;

use tauri::State;

use crate::application::session::commands::*;
use crate::domain::agent::runner::PermissionMode;
use crate::domain::project::value_objects::ProjectPath;
use crate::domain::task::value_objects::WorktreeRef;
use crate::infrastructure::AppServices;

fn load_env_vars(services: &AppServices) -> HashMap<String, String> {
    let config = services.project_service.get_project_config().ok();
    let disabled: std::collections::HashSet<String> = config
        .as_ref()
        .and_then(|c| c.disabled_env_var_keys.clone())
        .unwrap_or_default()
        .into_iter()
        .collect();
    let vars = config
        .and_then(|c| c.env_vars)
        .unwrap_or_default()
        .into_iter()
        .filter(|(k, _)| !disabled.contains(k))
        .collect();
    super::expand_env_values(vars)
}

fn load_claude_path(services: &AppServices) -> Option<String> {
    services
        .project_service
        .get_project_config()
        .ok()
        .and_then(|c| c.claude_path)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn run_claude_session(
    id: String,
    description: String,
    use_plan: bool,
    yolo: bool,
    claude_path: Option<String>,
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

    let effective_claude_path = claude_path.or_else(|| load_claude_path(&services));
    let env_vars = load_env_vars(&services);

    // Auto-apply worktree links before starting the session
    if let Some(ref wt_name) = worktree {
        if let Ok(config) = services.project_service.get_project_config() {
            let links = config.worktree_links.unwrap_or_default();
            if !links.is_empty() {
                if let Ok(Some(ref proj)) = services.project_service.get_project() {
                    if let Err(e) = services.worktree_port.apply_links(
                        &ProjectPath::new(proj.clone()),
                        &WorktreeRef::new(wt_name.clone()),
                        &links,
                    ) {
                        log::warn!("[cmd] apply_links failed for worktree {}: {}", wt_name, e);
                    }
                }
            }
        }
    }

    log::info!("[cmd] run_claude_session: id={} plan={} yolo={} model={:?}", id, use_plan, yolo, model);

    let cmd = StartSessionCommand {
        task_id: id.clone(),
        description,
        permission_mode,
        agent_path: effective_claude_path,
        worktree: worktree.map(WorktreeRef::new),
        project_path: project_path.map(ProjectPath::new),
        model,
        agent_name: None,
        env_vars,
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
    log::info!("[cmd] stop_claude_session: id={}", id);
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
    id: String,
    input: String,
    model: Option<String>,
    yolo: bool,
    services: State<'_, AppServices>,
) -> Result<(), String> {
    log::info!("[cmd] send_input: id={} input_len={} model={:?} yolo={}", id, input.len(), model, yolo);
    let env_vars = load_env_vars(&services);
    services
        .session_service
        .send_input(SendInputCommand {
            task_id: id,
            input,
            model,
            yolo,
            env_vars,
        })
        .map_err(|e| {
            log::error!("[cmd] send_input failed: {}", e);
            e.to_string()
        })
}

#[tauri::command]
pub fn get_output_buffer(
    id: String,
    services: State<'_, AppServices>,
) -> Result<Vec<String>, String> {
    Ok(services.session_service.get_output_buffer(&id))
}
