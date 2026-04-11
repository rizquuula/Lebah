use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;
use crate::domain::task::events::TaskDomainEvent;
use crate::domain::task::value_objects::{
    AgentConfig, ExecutionFlags, TaskColumn, TaskId, TaskStatus, WorktreeRef,
};

/// Task aggregate root — all mutations go through behavior methods.
/// Fields are private; external access is via read-only accessors only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    id: TaskId,
    description: String,
    column: TaskColumn,
    status: TaskStatus,
    agent_config: AgentConfig,
    execution_flags: ExecutionFlags,
    worktree: Option<WorktreeRef>,
    sort_order: i32,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    has_run: bool,
    lines_added: Option<i32>,
    lines_removed: Option<i32>,
}

impl Task {
    /// Factory — the only way to construct a Task.
    pub fn create(
        description: String,
        agent_config: AgentConfig,
        execution_flags: ExecutionFlags,
        worktree: Option<WorktreeRef>,
        sort_order: i32,
    ) -> Self {
        Task {
            id: TaskId::new(),
            description,
            column: TaskColumn::Todo,
            status: TaskStatus::Idle,
            agent_config,
            execution_flags,
            worktree,
            sort_order,
            created_at: Utc::now(),
            completed_at: None,
            has_run: false,
            lines_added: None,
            lines_removed: None,
        }
    }

    /// Reconstruct from persisted state (used by repository only).
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: TaskId,
        description: String,
        column: TaskColumn,
        status: TaskStatus,
        agent_config: AgentConfig,
        execution_flags: ExecutionFlags,
        worktree: Option<WorktreeRef>,
        sort_order: i32,
        created_at: DateTime<Utc>,
        completed_at: Option<DateTime<Utc>>,
        has_run: bool,
        lines_added: Option<i32>,
        lines_removed: Option<i32>,
    ) -> Self {
        Task {
            id,
            description,
            column,
            status,
            agent_config,
            execution_flags,
            worktree,
            sort_order,
            created_at,
            completed_at,
            has_run,
            lines_added,
            lines_removed,
        }
    }

    // --- Behavior methods ---

    pub fn move_to_column(
        &mut self,
        column: TaskColumn,
        sort_order: i32,
    ) -> Result<TaskDomainEvent, DomainError> {
        let from = self.column.clone();
        if column == TaskColumn::Completed {
            if self.completed_at.is_none() {
                self.completed_at = Some(Utc::now());
            }
            if self.status == TaskStatus::Running {
                self.status = TaskStatus::Success;
            }
        }
        self.column = column.clone();
        self.sort_order = sort_order;
        Ok(TaskDomainEvent::TaskMoved {
            task_id: self.id.clone(),
            from,
            to: column,
        })
    }

    pub fn mark_started(&mut self) -> Result<TaskDomainEvent, DomainError> {
        if self.status == TaskStatus::Running {
            return Err(DomainError::TaskAlreadyRunning);
        }
        self.status = TaskStatus::Running;
        self.has_run = true;
        Ok(TaskDomainEvent::TaskStarted {
            task_id: self.id.clone(),
        })
    }

    pub fn mark_completed(&mut self, success: bool) -> TaskDomainEvent {
        self.status = if success {
            TaskStatus::Success
        } else {
            TaskStatus::Failed
        };
        TaskDomainEvent::TaskCompleted {
            task_id: self.id.clone(),
            success,
        }
    }

    pub fn mark_stopped(&mut self) -> TaskDomainEvent {
        self.status = TaskStatus::Idle;
        TaskDomainEvent::TaskStopped {
            task_id: self.id.clone(),
        }
    }

    pub fn update_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn update_agent_config(&mut self, agent_config: AgentConfig) {
        self.agent_config = agent_config;
    }

    pub fn update_execution_flags(&mut self, flags: ExecutionFlags) {
        self.execution_flags = flags;
    }

    pub fn set_line_changes(&mut self, added: i32, removed: i32) {
        self.lines_added = Some(added);
        self.lines_removed = Some(removed);
    }

    // --- Read-only accessors ---

    pub fn id(&self) -> &TaskId {
        &self.id
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn column(&self) -> &TaskColumn {
        &self.column
    }
    pub fn status(&self) -> &TaskStatus {
        &self.status
    }
    pub fn agent_config(&self) -> &AgentConfig {
        &self.agent_config
    }
    pub fn execution_flags(&self) -> &ExecutionFlags {
        &self.execution_flags
    }
    pub fn worktree(&self) -> Option<&WorktreeRef> {
        self.worktree.as_ref()
    }
    pub fn sort_order(&self) -> i32 {
        self.sort_order
    }
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
    pub fn completed_at(&self) -> Option<&DateTime<Utc>> {
        self.completed_at.as_ref()
    }
    pub fn has_run(&self) -> bool {
        self.has_run
    }
    pub fn lines_added(&self) -> Option<i32> {
        self.lines_added
    }
    pub fn lines_removed(&self) -> Option<i32> {
        self.lines_removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::value_objects::{AgentConfig, ExecutionFlags, TaskColumn, TaskStatus};

    fn make_task() -> Task {
        Task::create(
            "test task".to_string(),
            AgentConfig::default(),
            ExecutionFlags::default(),
            None,
            0,
        )
    }

    #[test]
    fn create_sets_defaults() {
        let t = make_task();
        assert_eq!(t.description(), "test task");
        assert_eq!(t.column(), &TaskColumn::Todo);
        assert_eq!(t.status(), &TaskStatus::Idle);
        assert!(!t.has_run());
        assert!(t.completed_at().is_none());
    }

    #[test]
    fn mark_started_sets_running() {
        let mut t = make_task();
        t.mark_started().unwrap();
        assert_eq!(t.status(), &TaskStatus::Running);
        assert!(t.has_run());
    }

    #[test]
    fn mark_started_twice_returns_error() {
        let mut t = make_task();
        t.mark_started().unwrap();
        assert!(t.mark_started().is_err());
    }

    #[test]
    fn mark_completed_success() {
        let mut t = make_task();
        t.mark_started().unwrap();
        t.mark_completed(true);
        assert_eq!(t.status(), &TaskStatus::Success);
    }

    #[test]
    fn mark_completed_failure() {
        let mut t = make_task();
        t.mark_started().unwrap();
        t.mark_completed(false);
        assert_eq!(t.status(), &TaskStatus::Failed);
    }

    #[test]
    fn mark_stopped_resets_to_idle() {
        let mut t = make_task();
        t.mark_started().unwrap();
        t.mark_stopped();
        assert_eq!(t.status(), &TaskStatus::Idle);
    }

    #[test]
    fn move_to_completed_sets_completed_at() {
        let mut t = make_task();
        assert!(t.completed_at().is_none());
        t.move_to_column(TaskColumn::Completed, 1).unwrap();
        assert!(t.completed_at().is_some());
        assert_eq!(t.column(), &TaskColumn::Completed);
    }

    #[test]
    fn move_to_completed_does_not_overwrite_completed_at() {
        let mut t = make_task();
        t.move_to_column(TaskColumn::Completed, 1).unwrap();
        let first = *t.completed_at().unwrap();
        t.move_to_column(TaskColumn::Completed, 2).unwrap();
        assert_eq!(*t.completed_at().unwrap(), first);
    }

    #[test]
    fn set_line_changes() {
        let mut t = make_task();
        t.set_line_changes(10, 5);
        assert_eq!(t.lines_added(), Some(10));
        assert_eq!(t.lines_removed(), Some(5));
    }
}
