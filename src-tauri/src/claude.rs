use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub struct SessionConfig {
    pub use_plan: bool,
    pub yolo: bool,
    pub claude_path: Option<String>,
    pub claude_command: Option<String>,
    pub worktree: Option<String>,
    pub project_path: Option<String>,
}

pub struct SessionManager {
    pub sessions: Arc<Mutex<HashMap<String, Child>>>,
    pub output_buffers: Arc<Mutex<HashMap<String, Vec<String>>>>,
    pub session_configs: Mutex<HashMap<String, SessionConfig>>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            output_buffers: Arc::new(Mutex::new(HashMap::new())),
            session_configs: Mutex::new(HashMap::new()),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_session(
        &self,
        app: &AppHandle,
        task_id: &str,
        description: &str,
        use_plan: bool,
        yolo: bool,
        claude_path: Option<&str>,
        claude_command: Option<&str>,
        worktree: Option<&str>,
        project_path: Option<&str>,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;

        if sessions.contains_key(task_id) {
            return Err("Session already running for this task".to_string());
        }

        // Clear old buffer for this task
        {
            let mut buffers = self.output_buffers.lock().map_err(|e| e.to_string())?;
            buffers.insert(task_id.to_string(), Vec::new());
        }

        let binary = claude_path.unwrap_or("claude");
        eprintln!("[claude] Starting session for task_id={}", task_id);
        eprintln!("[claude] Binary: {}", binary);
        eprintln!("[claude] Project path: {:?}", project_path);
        eprintln!("[claude] use_plan={} yolo={} worktree={:?}", use_plan, yolo, worktree);
        eprintln!("[claude] Description: {}", description);

        let mut cmd = Command::new(binary);
        cmd.arg("--session-id").arg(task_id);

        if let Some(project) = project_path {
            cmd.current_dir(project);
        }

        if use_plan {
            cmd.arg("--permission-mode").arg("plan");
        }

        if yolo {
            cmd.env("IS_SANDBOX", "1");
            cmd.arg("--dangerously-skip-permissions");
        }

        if let Some(wt) = worktree {
            cmd.arg("--worktree").arg(wt);
        }

        if let Some(extra) = claude_command {
            for arg in extra.split_whitespace() {
                cmd.arg(arg);
            }
        }

        cmd.arg("--output-format").arg("stream-json");
        cmd.arg("--verbose");
        cmd.arg("--print").arg(description);
        cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

        eprintln!("[claude] Spawning: {} {:?}", cmd.get_program().to_string_lossy(),
            cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect::<Vec<_>>());

        let mut child = cmd.spawn().map_err(|e| {
            eprintln!("[claude] Failed to spawn process: {}", e);
            format!("Failed to spawn claude: {}", e)
        })?;
        eprintln!("[claude] Spawned PID: {}", child.id());

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdin = child.stdin.take();

        // Spawn stdout reader thread
        if let Some(stdout) = stdout {
            let task_id_clone = task_id.to_string();
            let app_clone = app.clone();
            let buffers = Arc::clone(&self.output_buffers);
            std::thread::spawn(move || {
                eprintln!("[claude:stdout] Reader thread started for task={}", task_id_clone);
                let thread_storage = crate::storage::Storage::new().ok();
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    eprintln!("[claude:stdout] {}", line);
                    if let Ok(mut b) = buffers.lock() {
                        b.entry(task_id_clone.clone()).or_default().push(line.clone());
                    }
                    if let Some(ref s) = thread_storage {
                        let _ = s.append_output_line(&task_id_clone, &line);
                    }
                    let _ = app_clone.emit(&format!("claude-output-{}", task_id_clone), &line);
                }
                eprintln!("[claude:stdout] Reader thread ended for task={}", task_id_clone);
            });
        }

        // Spawn stderr reader thread
        if let Some(stderr) = stderr {
            let task_id_clone = task_id.to_string();
            let app_clone = app.clone();
            let buffers = Arc::clone(&self.output_buffers);
            std::thread::spawn(move || {
                eprintln!("[claude:stderr] Reader thread started for task={}", task_id_clone);
                let thread_storage = crate::storage::Storage::new().ok();
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    eprintln!("[claude:stderr] {}", line);
                    if let Ok(mut b) = buffers.lock() {
                        b.entry(task_id_clone.clone()).or_default().push(line.clone());
                    }
                    if let Some(ref s) = thread_storage {
                        let _ = s.append_output_line(&task_id_clone, &line);
                    }
                    let _ = app_clone.emit(&format!("claude-output-{}", task_id_clone), &line);
                }
                eprintln!("[claude:stderr] Reader thread ended for task={}", task_id_clone);
            });
        }

        let _ = stdin; // stdin is null in --print mode; follow-ups use --resume

        sessions.insert(task_id.to_string(), child);

        // Save config for follow-up messages via --resume
        if let Ok(mut configs) = self.session_configs.lock() {
            configs.insert(task_id.to_string(), SessionConfig {
                use_plan,
                yolo,
                claude_path: claude_path.map(|s| s.to_string()),
                claude_command: claude_command.map(|s| s.to_string()),
                worktree: worktree.map(|s| s.to_string()),
                project_path: project_path.map(|s| s.to_string()),
            });
        }

        Ok(())
    }

    pub fn get_output_buffer(&self, task_id: &str) -> Result<Vec<String>, String> {
        let buffers = self.output_buffers.lock().map_err(|e| e.to_string())?;
        Ok(buffers.get(task_id).cloned().unwrap_or_default())
    }

    pub fn sessions_arc(&self) -> Arc<Mutex<HashMap<String, Child>>> {
        Arc::clone(&self.sessions)
    }

    pub fn send_input(&self, task_id: &str, input: &str, app: &AppHandle) -> Result<(), String> {
        // In --print mode, follow-up messages are sent by spawning a new
        // claude process with --resume and --session-id to continue the conversation.
        let configs = self.session_configs.lock().map_err(|e| e.to_string())?;
        let config = configs.get(task_id)
            .ok_or_else(|| "No session config found for this task".to_string())?;

        let binary = config.claude_path.as_deref().unwrap_or("claude");
        let mut cmd = Command::new(binary);
        cmd.arg("--continue").arg("--session-id").arg(task_id).arg("--fork-session");

        if let Some(ref project) = config.project_path {
            cmd.current_dir(project);
        }

        if config.use_plan {
            cmd.arg("--permission-mode").arg("plan");
        }

        if config.yolo {
            cmd.env("IS_SANDBOX", "1");
            cmd.arg("--dangerously-skip-permissions");
        }

        if let Some(ref wt) = config.worktree {
            cmd.arg("--worktree").arg(wt);
        }

        if let Some(ref extra) = config.claude_command {
            for arg in extra.split_whitespace() {
                cmd.arg(arg);
            }
        }

        cmd.arg("--output-format").arg("stream-json");
        cmd.arg("--verbose");
        cmd.arg("--print").arg(input);
        cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

        drop(configs);

        eprintln!("[claude:resume] Spawning follow-up for task={}: {}", task_id, input);

        let mut child = cmd.spawn().map_err(|e| {
            eprintln!("[claude:resume] Failed to spawn: {}", e);
            format!("Failed to spawn follow-up claude: {}", e)
        })?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Stream stdout
        if let Some(stdout) = stdout {
            let tid = task_id.to_string();
            let app_c = app.clone();
            let buffers = Arc::clone(&self.output_buffers);
            std::thread::spawn(move || {
                let thread_storage = crate::storage::Storage::new().ok();
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    eprintln!("[claude:resume:stdout] {}", line);
                    if let Ok(mut b) = buffers.lock() {
                        b.entry(tid.clone()).or_default().push(line.clone());
                    }
                    if let Some(ref s) = thread_storage {
                        let _ = s.append_output_line(&tid, &line);
                    }
                    let _ = app_c.emit(&format!("claude-output-{}", tid), &line);
                }
            });
        }

        // Stream stderr
        if let Some(stderr) = stderr {
            let tid = task_id.to_string();
            let app_c = app.clone();
            let buffers = Arc::clone(&self.output_buffers);
            std::thread::spawn(move || {
                let thread_storage = crate::storage::Storage::new().ok();
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    eprintln!("[claude:resume:stderr] {}", line);
                    if let Ok(mut b) = buffers.lock() {
                        b.entry(tid.clone()).or_default().push(line.clone());
                    }
                    if let Some(ref s) = thread_storage {
                        let _ = s.append_output_line(&tid, &line);
                    }
                    let _ = app_c.emit(&format!("claude-output-{}", tid), &line);
                }
            });
        }

        // Store the follow-up process so it can be monitored/stopped
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions.insert(task_id.to_string(), child);

        Ok(())
    }

    pub fn stop_session(&self, task_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        if let Some(mut child) = sessions.remove(task_id) {
            let _ = child.kill();
        }
        Ok(())
    }
}
