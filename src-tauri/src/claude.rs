use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

pub struct SessionManager {
    pub sessions: Mutex<HashMap<String, Child>>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    pub fn run_session(
        &self,
        app: &AppHandle,
        task_id: &str,
        description: &str,
        use_plan: bool,
        yolo: bool,
        claude_path: Option<&str>,
        claude_command: Option<&str>,
        project_path: Option<&str>,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;

        if sessions.contains_key(task_id) {
            return Err("Session already running for this task".to_string());
        }

        let binary = claude_path.unwrap_or("claude");
        let mut cmd = Command::new(binary);
        cmd.arg("--session-id").arg(task_id);

        if let Some(project) = project_path {
            cmd.current_dir(project);
        }

        if use_plan {
            cmd.arg("--plan");
        }

        if yolo {
            cmd.arg("--dangerously-skip-permissions");
        }

        // Add custom command/args if provided
        if let Some(extra) = claude_command {
            for arg in extra.split_whitespace() {
                cmd.arg(arg);
            }
        }

        cmd.arg("--message").arg(description);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn claude: {}", e))?;

        let stdout = child.stdout.take();
        let task_id_clone = task_id.to_string();
        let app_clone = app.clone();

        if let Some(stdout) = stdout {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let event_name = format!("claude-output-{}", task_id_clone);
                        let _ = app_clone.emit(&event_name, &line);
                    }
                }
            });
        }

        sessions.insert(task_id.to_string(), child);
        Ok(())
    }

    pub fn stop_session(&self, task_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        if let Some(mut child) = sessions.remove(task_id) {
            child.kill().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("No session running for this task".to_string())
        }
    }

}
