use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub project: String,
}

struct PtySession {
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    writer: Option<Box<dyn Write + Send>>,
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    reader_running: Arc<Mutex<bool>>,
}

impl PtySession {
    fn kill(&mut self) {
        if let Ok(mut running) = self.reader_running.lock() {
            *running = false;
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
        self.writer = None;
        self.master = None;
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        self.kill();
    }
}

pub struct PtySessionManager {
    sessions: HashMap<String, PtySession>,
    session_info: HashMap<String, SessionInfo>,
    counter: u32,
}

impl PtySessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            session_info: HashMap::new(),
            counter: 0,
        }
    }

    pub fn create_session(
        &mut self,
        cwd: &str,
        cols: u16,
        rows: u16,
        app_handle: AppHandle,
    ) -> Result<SessionInfo, String> {
        let id = Uuid::new_v4().to_string();
        self.counter += 1;
        let name = format!("Session {}", self.counter);

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let shell = Self::default_shell();
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(cwd);

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone PTY reader: {}", e))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take PTY writer: {}", e))?;

        let running = Arc::new(Mutex::new(true));
        let running_clone = Arc::clone(&running);
        let event_name = format!("terminal-output-{}", id);

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                {
                    let is_running = running_clone.lock().unwrap();
                    if !*is_running {
                        break;
                    }
                }
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = app_handle.emit(&event_name, text);
                    }
                    Err(_) => break,
                }
            }
        });

        let session = PtySession {
            child: Some(child),
            writer: Some(writer),
            master: Some(pair.master),
            reader_running: running,
        };

        let info = SessionInfo {
            id: id.clone(),
            name: name.clone(),
            project: cwd.to_string(),
        };

        self.sessions.insert(id.clone(), session);
        self.session_info.insert(id, info.clone());

        Ok(info)
    }

    pub fn write(&mut self, session_id: &str, data: &str) -> Result<(), String> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or("Session not found")?;
        let writer = session.writer.as_mut().ok_or("No terminal running")?;
        writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("Write failed: {}", e))?;
        writer.flush().map_err(|e| format!("Flush failed: {}", e))
    }

    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let session = self.sessions.get(session_id).ok_or("Session not found")?;
        let master = session.master.as_ref().ok_or("No terminal running")?;
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Resize failed: {}", e))
    }

    pub fn close_session(&mut self, session_id: &str) {
        if let Some(mut session) = self.sessions.remove(session_id) {
            session.kill();
        }
        self.session_info.remove(session_id);
    }

    pub fn list_sessions(&self, project: Option<&str>) -> Vec<SessionInfo> {
        self.session_info
            .values()
            .filter(|info| match project {
                Some(p) => info.project == p,
                None => true,
            })
            .cloned()
            .collect()
    }

    pub fn kill_all(&mut self) {
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            self.close_session(&id);
        }
        self.counter = 0;
    }

    fn default_shell() -> String {
        #[cfg(unix)]
        {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        }
        #[cfg(windows)]
        {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        }
    }
}

impl Drop for PtySessionManager {
    fn drop(&mut self) {
        self.kill_all();
    }
}
