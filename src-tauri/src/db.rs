use rusqlite::{Connection, params};
use std::sync::Mutex;

use crate::models::{Task, TaskColumn, TaskStatus};

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: &std::path::Path) -> Result<Self, String> {
        std::fs::create_dir_all(app_dir).map_err(|e| e.to_string())?;
        let db_path = app_dir.join("lebahtempa.db");
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                column_name TEXT NOT NULL DEFAULT 'Todo',
                status TEXT NOT NULL DEFAULT 'Idle',
                use_plan INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );"
        ).map_err(|e| e.to_string())?;

        // Migration: add claude_path and claude_command columns
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(tasks)")
            .map_err(|e| e.to_string())?
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        if !columns.contains(&"claude_path".to_string()) {
            conn.execute_batch("ALTER TABLE tasks ADD COLUMN claude_path TEXT;")
                .map_err(|e| e.to_string())?;
        }
        if !columns.contains(&"claude_command".to_string()) {
            conn.execute_batch("ALTER TABLE tasks ADD COLUMN claude_command TEXT;")
                .map_err(|e| e.to_string())?;
        }
        if !columns.contains(&"yolo".to_string()) {
            conn.execute_batch("ALTER TABLE tasks ADD COLUMN yolo INTEGER NOT NULL DEFAULT 1;")
                .map_err(|e| e.to_string())?;
        }
        if !columns.contains(&"worktree".to_string()) {
            conn.execute_batch("ALTER TABLE tasks ADD COLUMN worktree TEXT;")
                .map_err(|e| e.to_string())?;
        }

        Ok(Database { conn: Mutex::new(conn) })
    }

    pub fn get_tasks(&self) -> Result<Vec<Task>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, description, column_name, status, use_plan, sort_order, created_at, claude_path, claude_command, yolo, worktree FROM tasks ORDER BY sort_order")
            .map_err(|e| e.to_string())?;

        let tasks = stmt
            .query_map([], |row| {
                let column_str: String = row.get(2)?;
                let status_str: String = row.get(3)?;
                Ok(Task {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    column: TaskColumn::from_str(&column_str).unwrap_or(TaskColumn::Todo),
                    status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Idle),
                    use_plan: row.get::<_, i32>(4)? != 0,
                    sort_order: row.get(5)?,
                    created_at: row.get(6)?,
                    claude_path: row.get(7)?,
                    claude_command: row.get(8)?,
                    yolo: row.get::<_, i32>(9)? != 0,
                    worktree: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        Ok(tasks)
    }

    pub fn create_task(&self, id: &str, description: &str, created_at: &str, claude_path: Option<&str>, claude_command: Option<&str>, worktree: Option<&str>) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO tasks (id, description, column_name, status, use_plan, yolo, sort_order, created_at, claude_path, claude_command, worktree) VALUES (?1, ?2, 'Todo', 'Idle', 0, 1, 0, ?3, ?4, ?5, ?6)",
            params![id, description, created_at, claude_path, claude_command, worktree],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_task(&self, task: &Task) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE tasks SET description = ?1, column_name = ?2, status = ?3, use_plan = ?4, yolo = ?5, sort_order = ?6, claude_path = ?7, claude_command = ?8 WHERE id = ?9",
            params![
                task.description,
                task.column.as_str(),
                task.status.as_str(),
                task.use_plan as i32,
                task.yolo as i32,
                task.sort_order,
                task.claude_path,
                task.claude_command,
                task.id,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_task_worktree(&self, id: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT worktree FROM tasks WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        let worktree = stmt
            .query_row(params![id], |row| row.get::<_, Option<String>>(0))
            .map_err(|e| e.to_string())?;
        Ok(worktree)
    }

    pub fn delete_task(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn move_task(&self, id: &str, column: &str, sort_order: i32) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE tasks SET column_name = ?1, sort_order = ?2 WHERE id = ?3",
            params![column, sort_order, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_task_status(&self, id: &str, status: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            params![status, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}
