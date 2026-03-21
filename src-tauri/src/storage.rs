use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::{GlobalConfig, Task};

pub struct Storage {
    base_dir: PathBuf,
    pub current_project: Mutex<Option<String>>,
}

impl Storage {
    pub fn new() -> Result<Self, String> {
        let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
        let base_dir = home.join(".lebahcode");
        std::fs::create_dir_all(base_dir.join("projects")).map_err(|e| e.to_string())?;

        let storage = Storage {
            base_dir,
            current_project: Mutex::new(None),
        };

        // Load last project from config
        let config = storage.load_config();
        if let Some(ref path) = config.last_project {
            if std::path::Path::new(path).is_dir() {
                *storage.current_project.lock().map_err(|e| e.to_string())? = Some(path.clone());
            }
        }

        Ok(storage)
    }

    fn project_hash(project_path: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(project_path.as_bytes());
        let result = hasher.finalize();
        result[..8].iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn project_dir(&self, project_path: &str) -> PathBuf {
        self.base_dir.join("projects").join(Self::project_hash(project_path))
    }

    fn require_project(&self) -> Result<String, String> {
        self.current_project
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .ok_or_else(|| "No project path set".to_string())
    }

    // --- Config ---

    pub fn load_config(&self) -> GlobalConfig {
        let path = self.base_dir.join("config.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_config(&self, config: &GlobalConfig) -> Result<(), String> {
        let path = self.base_dir.join("config.json");
        let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    // --- Project ---

    pub fn set_project(&self, project_path: &str) -> Result<(), String> {
        let dir = self.project_dir(project_path);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

        // Write project.json
        let meta = serde_json::json!({ "path": project_path });
        let meta_path = dir.join("project.json");
        std::fs::write(meta_path, serde_json::to_string_pretty(&meta).unwrap())
            .map_err(|e| e.to_string())?;

        // Update config
        let mut config = self.load_config();
        config.last_project = Some(project_path.to_string());
        self.save_config(&config)?;

        // Update in-memory
        *self.current_project.lock().map_err(|e| e.to_string())? = Some(project_path.to_string());
        Ok(())
    }

    pub fn get_project(&self) -> Result<Option<String>, String> {
        self.current_project.lock().map_err(|e| e.to_string()).map(|p| p.clone())
    }

    // --- Tasks ---

    fn load_tasks_for(&self, project_path: &str) -> Vec<Task> {
        let path = self.project_dir(project_path).join("tasks.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_tasks_for(&self, project_path: &str, tasks: &[Task]) -> Result<(), String> {
        let dir = self.project_dir(project_path);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(tasks).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("tasks.json"), json).map_err(|e| e.to_string())
    }

    pub fn get_tasks(&self) -> Result<Vec<Task>, String> {
        let project = match self.current_project.lock().map_err(|e| e.to_string())?.clone() {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };
        let mut tasks = self.load_tasks_for(&project);
        tasks.sort_by_key(|t| t.sort_order);
        Ok(tasks)
    }

    pub fn create_task(
        &self,
        id: &str,
        description: &str,
        created_at: &str,
        claude_path: Option<&str>,
        claude_command: Option<&str>,
        worktree: Option<&str>,
    ) -> Result<(), String> {
        let project = self.require_project()?;
        let mut tasks = self.load_tasks_for(&project);
        tasks.push(Task {
            id: id.to_string(),
            description: description.to_string(),
            column: crate::models::TaskColumn::Todo,
            status: crate::models::TaskStatus::Idle,
            use_plan: false,
            yolo: true,
            sort_order: 0,
            created_at: created_at.to_string(),
            claude_path: claude_path.map(|s| s.to_string()),
            claude_command: claude_command.map(|s| s.to_string()),
            worktree: worktree.map(|s| s.to_string()),
            has_run: false,
        });
        self.save_tasks_for(&project, &tasks)
    }

    pub fn update_task(&self, task: &Task) -> Result<(), String> {
        let project = self.require_project()?;
        let mut tasks = self.load_tasks_for(&project);
        if let Some(t) = tasks.iter_mut().find(|t| t.id == task.id) {
            t.description = task.description.clone();
            t.column = task.column.clone();
            t.status = task.status.clone();
            t.use_plan = task.use_plan;
            t.yolo = task.yolo;
            t.sort_order = task.sort_order;
            t.claude_path = task.claude_path.clone();
            t.claude_command = task.claude_command.clone();
        }
        self.save_tasks_for(&project, &tasks)
    }

    pub fn delete_task(&self, id: &str) -> Result<(), String> {
        let project = self.require_project()?;
        let mut tasks = self.load_tasks_for(&project);
        tasks.retain(|t| t.id != id);
        self.save_tasks_for(&project, &tasks)
    }

    pub fn move_task(&self, id: &str, column: &str, sort_order: i32) -> Result<(), String> {
        let project = self.require_project()?;
        let mut tasks = self.load_tasks_for(&project);
        if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
            t.column = crate::models::TaskColumn::from_str(column)?;
            t.sort_order = sort_order;
        }
        self.save_tasks_for(&project, &tasks)
    }

    pub fn update_task_status(&self, id: &str, status: &str) -> Result<(), String> {
        let project = self.require_project()?;
        let mut tasks = self.load_tasks_for(&project);
        if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
            t.status = crate::models::TaskStatus::from_str(status)?;
        }
        self.save_tasks_for(&project, &tasks)
    }

    pub fn get_task_worktree(&self, id: &str) -> Result<Option<String>, String> {
        let project = self.require_project()?;
        let tasks = self.load_tasks_for(&project);
        Ok(tasks.iter().find(|t| t.id == id).and_then(|t| t.worktree.clone()))
    }

    pub fn get_task(&self, id: &str) -> Result<crate::models::Task, String> {
        let project = self.require_project()?;
        let tasks = self.load_tasks_for(&project);
        tasks.into_iter().find(|t| t.id == id).ok_or_else(|| format!("Task not found: {}", id))
    }

    pub fn set_task_has_run(&self, id: &str, has_run: bool) -> Result<(), String> {
        let project = self.require_project()?;
        let mut tasks = self.load_tasks_for(&project);
        if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
            t.has_run = has_run;
        }
        self.save_tasks_for(&project, &tasks)
    }

    pub fn set_task_settings(&self, id: &str, use_plan: bool, yolo: bool) -> Result<(), String> {
        let project = self.require_project()?;
        let mut tasks = self.load_tasks_for(&project);
        if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
            t.use_plan = use_plan;
            t.yolo = yolo;
        }
        self.save_tasks_for(&project, &tasks)
    }

    /// For use from monitoring thread — takes explicit project path
    pub fn update_task_status_for(
        &self,
        project_path: &str,
        id: &str,
        status: &str,
    ) -> Result<(), String> {
        let mut tasks = self.load_tasks_for(project_path);
        if let Some(t) = tasks.iter_mut().find(|t| t.id == id) {
            t.status = crate::models::TaskStatus::from_str(status)?;
        }
        self.save_tasks_for(project_path, &tasks)
    }
}
