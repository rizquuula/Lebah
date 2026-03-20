use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskColumn {
    Todo,
    InProgress,
    Review,
    Merge,
    Completed,
}

impl TaskColumn {
    pub fn as_str(&self) -> &str {
        match self {
            TaskColumn::Todo => "Todo",
            TaskColumn::InProgress => "InProgress",
            TaskColumn::Review => "Review",
            TaskColumn::Merge => "Merge",
            TaskColumn::Completed => "Completed",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Todo" => Ok(TaskColumn::Todo),
            "InProgress" => Ok(TaskColumn::InProgress),
            "Review" => Ok(TaskColumn::Review),
            "Merge" => Ok(TaskColumn::Merge),
            "Completed" => Ok(TaskColumn::Completed),
            _ => Err(format!("Unknown column: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Idle,
    Running,
    Success,
    Failed,
    Warning,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Idle => "Idle",
            TaskStatus::Running => "Running",
            TaskStatus::Success => "Success",
            TaskStatus::Failed => "Failed",
            TaskStatus::Warning => "Warning",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Idle" => Ok(TaskStatus::Idle),
            "Running" => Ok(TaskStatus::Running),
            "Success" => Ok(TaskStatus::Success),
            "Failed" => Ok(TaskStatus::Failed),
            "Warning" => Ok(TaskStatus::Warning),
            _ => Err(format!("Unknown status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub column: TaskColumn,
    pub status: TaskStatus,
    pub use_plan: bool,
    pub yolo: bool,
    pub sort_order: i32,
    pub created_at: String,
    pub claude_path: Option<String>,
    pub claude_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub changed_files: u32,
}

pub struct ProjectState {
    pub path: Mutex<Option<String>>,
}

