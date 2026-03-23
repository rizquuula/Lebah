#[tauri::command]
pub fn check_path_exists(path: String) -> bool {
    std::path::Path::new(&path).exists()
}

#[tauri::command]
pub fn generate_worktree_name(
    description: String,
    claude_path: Option<String>,
) -> Result<String, String> {
    let claude = claude_path.unwrap_or_else(|| "claude".to_string());
    let prompt = format!(
        "Based on this task\n\n{}\n\nplease generate a worktree name, with format\n\n<fix/feat/chore>-<worktree name max 2 word separated by dash>-<5 random string character>\n\nRespond with ONLY the worktree name, nothing else. No explanation, no punctuation, just the name.",
        description
    );

    let output = std::process::Command::new(&claude)
        .arg("-p")
        .arg(&prompt)
        .arg("--model")
        .arg("claude-haiku-4-5-20251001")
        .output()
        .map_err(|e| format!("Failed to run claude: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Claude failed: {}", stderr));
    }

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(result)
}
