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

fn load_opencode_path(services: &AppServices) -> Option<String> {
    services
        .project_service
        .get_project_config()
        .ok()
        .and_then(|c| c.opencode_path)
}

fn load_agent_path(services: &AppServices, agent_name: Option<&str>) -> Option<String> {
    match agent_name {
        Some("opencode") => load_opencode_path(services),
        _ => load_claude_path(services),
    }
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn run_agent_session(
    id: String,
    description: String,
    use_plan: bool,
    yolo: bool,
    claude_path: Option<String>,
    worktree: Option<String>,
    model: Option<String>,
    agent_name: Option<String>,
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

    let effective_agent_path = claude_path.or_else(|| load_agent_path(&services, agent_name.as_deref()));
    let env_vars = load_env_vars(&services);

    // Collect worktree links to be applied after Claude CLI creates the worktree dir
    let worktree_links = if worktree.is_some() {
        services
            .project_service
            .get_project_config()
            .ok()
            .and_then(|c| c.worktree_links)
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    log::info!("[cmd] run_agent_session: id={} agent={:?} plan={} yolo={} model={:?}", id, agent_name, use_plan, yolo, model);

    let cmd = StartSessionCommand {
        task_id: id.clone(),
        description,
        permission_mode,
        agent_path: effective_agent_path,
        worktree: worktree.map(WorktreeRef::new),
        project_path: project_path.map(ProjectPath::new),
        model,
        agent_name,
        env_vars,
        worktree_links,
    };

    services.session_service.start_session(cmd).map_err(|e| {
        log::error!("[cmd] run_agent_session failed for {}: {}", id, e);
        e.to_string()
    })
}

#[tauri::command]
pub fn list_agents(
    services: State<'_, AppServices>,
) -> Result<Vec<String>, String> {
    Ok(services.session_service.list_agents())
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
