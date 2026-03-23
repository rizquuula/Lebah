use tauri::Emitter;

#[tauri::command]
pub fn check_path_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

#[tauri::command]
pub async fn generate_worktree_name(
    description: String,
    model: Option<String>,
    claude_path: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let binary = claude_path.as_deref().unwrap_or("claude");

        let prompt = format!(
            "Generate a concise git branch/worktree name for this task. \
            Rules: kebab-case only, max 30 characters, lowercase, hyphens only (no underscores or spaces), \
            use a short prefix like feat-, fix-, or chore- based on context. \
            Respond with ONLY the branch name, nothing else, no explanation.\n\nTask: {}",
            description
        );

        let mut cmd = std::process::Command::new(binary);
        cmd.arg("--output-format").arg("stream-json")
            .arg("--verbose")
            .arg("--print").arg(&prompt);

        if let Some(ref m) = model {
            cmd.arg("--model").arg(m);
        }

        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn claude: {}", e))?;

        let stdout = child.stdout.take().ok_or("No stdout")?;
        let reader = std::io::BufReader::new(stdout);

        let mut result_text = String::new();

        use std::io::BufRead;
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
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

        child.wait().map_err(|e| e.to_string())?;

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
    })
    .await
    .map_err(|e| e.to_string())?
}
