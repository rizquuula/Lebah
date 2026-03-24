use tauri::{Emitter, State};

use crate::infrastructure::AppServices;

#[tauri::command]
pub fn check_path_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

#[tauri::command]
pub fn get_app_version(app_handle: tauri::AppHandle) -> String {
    app_handle.package_info().version.to_string()
}

#[tauri::command]
pub async fn generate_worktree_name(
    description: String,
    model: Option<String>,
    claude_path: Option<String>,
    app_handle: tauri::AppHandle,
    services: State<'_, AppServices>,
) -> Result<String, String> {
    let config = services.project_service.get_project_config().ok();

    let claude = claude_path
        .or_else(|| config.as_ref().and_then(|c| c.claude_path.clone()))
        .unwrap_or_else(|| "claude".to_string());

    let effective_model = model
        .or_else(|| config.as_ref().and_then(|c| c.worktree_model.clone()));

    let disabled: std::collections::HashSet<String> = config
        .as_ref()
        .and_then(|c| c.disabled_env_var_keys.clone())
        .unwrap_or_default()
        .into_iter()
        .collect();
    let env_vars: std::collections::HashMap<String, String> = config
        .and_then(|c| c.env_vars)
        .unwrap_or_default()
        .into_iter()
        .filter(|(k, _)| !disabled.contains(k))
        .collect();

    let prompt = format!(
        "Based on this task\n\n{}\n\nplease generate a worktree name, with format\n\n<fix/feat/chore>-<worktree name max 2 word separated by dash>-<5 random string character>\n\nRespond with ONLY the worktree name, nothing else. No explanation, no punctuation, just the name.",
        description
    );

    let mut cmd = tokio::process::Command::new(&claude);
    cmd.arg("--output-format").arg("stream-json")
        .arg("--verbose")
        .arg("--print").arg(&prompt);

    if let Some(ref m) = effective_model {
        cmd.arg("--model").arg(m);
    }

    cmd.envs(&env_vars);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to run claude: {}", e))?;

    let stdout = child.stdout.take().ok_or("No stdout")?;
    let mut lines = tokio::io::BufReader::new(stdout).lines();

    let mut result_text = String::new();

    use tokio::io::AsyncBufReadExt;
    while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
        if line.trim().is_empty() {
            continue;
        }
        app_handle.emit("worktree-gen-line", &line).ok();

        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&line) {
            if obj["type"] == "assistant" {
                if let Some(content) = obj["message"]["content"].as_array() {
                    for part in content {
                        if part["type"] == "text" {
                            if let Some(text) = part["text"].as_str() {
                                result_text += text;
                            }
                        }
                    }
                }
            }
        }
    }

    child.wait().await.map_err(|e| e.to_string())?;

    let name: String = result_text
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if name.is_empty() {
        return Err("Claude did not return a valid name".to_string());
    }

    let name: String = name.chars().take(50).collect();
    Ok(name)
}
