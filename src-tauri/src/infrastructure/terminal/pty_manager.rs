use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub struct PtyManager {
    child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    writer: Option<Box<dyn Write + Send>>,
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    reader_running: Arc<Mutex<bool>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            child: None,
            writer: None,
            master: None,
            reader_running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn spawn(&mut self, cwd: &str, cols: u16, rows: u16, app_handle: AppHandle) -> Result<(), String> {
        self.kill();

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
                        let _ = app_handle.emit("terminal-output", text);
                    }
                    Err(_) => break,
                }
            }
        });

        self.child = Some(child);
        self.writer = Some(writer);
        self.master = Some(pair.master);
        self.reader_running = running;

        Ok(())
    }

    pub fn write(&mut self, data: &str) -> Result<(), String> {
        let writer = self.writer.as_mut().ok_or("No terminal running")?;
        writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("Write failed: {}", e))?;
        writer.flush().map_err(|e| format!("Flush failed: {}", e))
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        let master = self.master.as_ref().ok_or("No terminal running")?;
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Resize failed: {}", e))
    }

    pub fn kill(&mut self) {
        if let Ok(mut running) = self.reader_running.lock() {
            *running = false;
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
        }
        self.writer = None;
        self.master = None;
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

impl Drop for PtyManager {
    fn drop(&mut self) {
        self.kill();
    }
}
