use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;
use crate::domain::project::value_objects::ProjectId;
use crate::domain::repositories::TaskRepository;
use crate::domain::task::aggregate::Task;
use crate::domain::task::value_objects::{
    AgentConfig, ExecutionFlags, TaskColumn, TaskId, TaskStatus, WorktreeRef,
};
use crate::infrastructure::persistence::path_resolver::PathResolver;

/// JSON serialization DTO for Task — decoupled from the domain aggregate
#[derive(Debug, Serialize, Deserialize)]
struct TaskRecord {
    id: String,
    description: String,
    column: String,
    status: String,
    use_plan: bool,
    yolo: bool,
    #[serde(default)]
    auto: bool,
    sort_order: i32,
    created_at: String,
    #[serde(default)]
    completed_at: Option<String>,
    claude_path: Option<String>,
    #[serde(default)]
    claude_command: Option<String>,
    worktree: Option<String>,
    #[serde(default)]
    has_run: bool,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    agent_name: Option<String>,
    #[serde(default)]
    lines_added: Option<i32>,
    #[serde(default)]
    lines_removed: Option<i32>,
}

impl TaskRecord {
    fn from_task(task: &Task) -> Self {
        Self {
            id: task.id().to_string(),
            description: task.description().to_string(),
            column: task.column().as_str().to_string(),
            status: task.status().as_str().to_string(),
            use_plan: task.execution_flags().use_plan,
            yolo: task.execution_flags().yolo,
            auto: task.execution_flags().auto,
            sort_order: task.sort_order(),
            created_at: task.created_at().to_rfc3339(),
            completed_at: task.completed_at().map(|dt| dt.to_rfc3339()),
            claude_path: task.agent_config().agent_path.clone(),
            claude_command: None,
            worktree: task.worktree().map(|w| w.0.clone()),
            has_run: task.has_run(),
            model: task.agent_config().model.clone(),
            agent_name: task.agent_config().agent_name.clone(),
            lines_added: task.lines_added(),
            lines_removed: task.lines_removed(),
        }
    }

    fn into_task(self) -> Result<Task, DomainError> {
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());
        let completed_at = self.completed_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        });
        Ok(Task::reconstitute(
            TaskId::from_string(self.id),
            self.description,
            TaskColumn::from_str(&self.column)?,
            TaskStatus::from_str(&self.status)?,
            AgentConfig {
                agent_name: self.agent_name,
                agent_path: self.claude_path,
                model: self.model,
            },
            ExecutionFlags {
                use_plan: self.use_plan,
                yolo: self.yolo,
                auto: self.auto,
            },
            self.worktree.map(WorktreeRef::new),
            self.sort_order,
            created_at,
            completed_at,
            self.has_run,
            self.lines_added,
            self.lines_removed,
        ))
    }
}

pub struct JsonTaskRepository {
    resolver: Arc<PathResolver>,
}

impl JsonTaskRepository {
    pub fn new(resolver: Arc<PathResolver>) -> Self {
        Self { resolver }
    }

    fn load_records(&self, project_id: &ProjectId) -> Vec<TaskRecord> {
        let path = self.resolver.tasks_path(project_id);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_records(
        &self,
        project_id: &ProjectId,
        records: &[TaskRecord],
    ) -> Result<(), DomainError> {
        let dir = self.resolver.project_dir(project_id);
        std::fs::create_dir_all(&dir).map_err(|e| DomainError::InvalidValue(e.to_string()))?;
        let json = serde_json::to_string_pretty(records)
            .map_err(|e| DomainError::InvalidValue(e.to_string()))?;
        std::fs::write(self.resolver.tasks_path(project_id), json)
            .map_err(|e| DomainError::InvalidValue(e.to_string()))
    }
}

impl TaskRepository for JsonTaskRepository {
    fn find_all(&self, project_id: &ProjectId) -> Result<Vec<Task>, DomainError> {
        let mut records = self.load_records(project_id);
        records.sort_by_key(|r| r.sort_order);
        records.into_iter().map(|r| r.into_task()).collect()
    }

    fn find_by_id(&self, project_id: &ProjectId, task_id: &TaskId) -> Result<Task, DomainError> {
        let records = self.load_records(project_id);
        records
            .into_iter()
            .find(|r| r.id == task_id.0)
            .ok_or_else(|| DomainError::TaskNotFound(task_id.to_string()))?
            .into_task()
    }

    fn save(&self, project_id: &ProjectId, task: &Task) -> Result<(), DomainError> {
        let mut records = self.load_records(project_id);
        let record = TaskRecord::from_task(task);
        if let Some(existing) = records.iter_mut().find(|r| r.id == task.id().0) {
            *existing = record;
        } else {
            records.push(record);
        }
        self.save_records(project_id, &records)
    }

    fn delete(&self, project_id: &ProjectId, task_id: &TaskId) -> Result<(), DomainError> {
        let mut records = self.load_records(project_id);
        records.retain(|r| r.id != task_id.0);
        self.save_records(project_id, &records)
    }
}
