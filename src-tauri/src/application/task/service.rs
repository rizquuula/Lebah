use std::sync::Arc;

use crate::application::errors::ApplicationError;
use crate::application::event_bus::{DomainEvent, DomainEventBus};
use crate::application::ports::WorktreePort;
use crate::application::task::commands::*;
use crate::domain::errors::DomainError;
use crate::domain::project::value_objects::ProjectId;
use crate::domain::repositories::{OutputRepository, TaskRepository};
use crate::domain::task::aggregate::Task;
use crate::domain::task::events::TaskDomainEvent;
use crate::domain::task::value_objects::{
    AgentConfig, ExecutionFlags, TaskColumn, TaskId, TaskStatus,
};

pub struct TaskApplicationService {
    task_repo: Arc<dyn TaskRepository>,
    output_repo: Arc<dyn OutputRepository>,
    worktree_port: Arc<dyn WorktreePort>,
    event_bus: Arc<dyn DomainEventBus>,
    current_project: Arc<std::sync::Mutex<Option<String>>>,
}

impl TaskApplicationService {
    pub fn new(
        task_repo: Arc<dyn TaskRepository>,
        output_repo: Arc<dyn OutputRepository>,
        worktree_port: Arc<dyn WorktreePort>,
        event_bus: Arc<dyn DomainEventBus>,
        current_project: Arc<std::sync::Mutex<Option<String>>>,
    ) -> Self {
        Self {
            task_repo,
            output_repo,
            worktree_port,
            event_bus,
            current_project,
        }
    }

    fn current_project_id(&self) -> Result<(ProjectId, String), ApplicationError> {
        let path = self
            .current_project
            .lock()
            .map_err(|e| ApplicationError::Persistence(e.to_string()))?
            .clone()
            .ok_or(ApplicationError::Domain(DomainError::NoProjectSelected))?;
        Ok((ProjectId::from_path(&path), path))
    }

    pub fn get_tasks(&self) -> Result<Vec<Task>, ApplicationError> {
        let guard = self.current_project.lock()
            .map_err(|e| ApplicationError::Persistence(e.to_string()))?;
        let Some(ref path) = *guard else {
            return Ok(Vec::new());
        };
        let project_id = ProjectId::from_path(path);
        drop(guard);
        Ok(self.task_repo.find_all(&project_id)?)
    }

    pub fn create_task(&self, cmd: CreateTaskCommand) -> Result<Task, ApplicationError> {
        log::info!("[task] Creating task: desc_len={}", cmd.description.len());
        let (project_id, _) = self.current_project_id()?;
        let task = Task::create(
            cmd.description,
            cmd.agent_config,
            cmd.execution_flags,
            cmd.worktree,
            cmd.sort_order,
        );
        self.task_repo.save(&project_id, &task)?;
        self.event_bus.publish(DomainEvent::Task(TaskDomainEvent::TaskCreated {
            task_id: task.id().clone(),
            project_id,
        }));
        Ok(task)
    }

    pub fn update_task(&self, cmd: UpdateTaskCommand) -> Result<(), ApplicationError> {
        let (project_id, _) = self.current_project_id()?;
        let task_id = TaskId::from_string(cmd.id);
        let mut task = self.task_repo.find_by_id(&project_id, &task_id)?;
        task.update_description(cmd.description);
        task.update_agent_config(AgentConfig {
            agent_name: None,
            agent_path: cmd.agent_path,

            model: cmd.model,
        });
        task.update_execution_flags(ExecutionFlags {
            use_plan: cmd.use_plan,
            yolo: cmd.yolo,
        });
        // For direct status/column updates (e.g. from frontend), bypass aggregate invariants
        // by reconstituting with new values
        let column = TaskColumn::from_str(&cmd.column)?;
        let status = TaskStatus::from_str(&cmd.status)?;
        let completed_at = if column == TaskColumn::Completed {
            task.completed_at().cloned().or_else(|| Some(chrono::Utc::now()))
        } else {
            task.completed_at().cloned()
        };
        let new_task = Task::reconstitute(
            task_id,
            task.description().to_string(),
            column,
            status,
            task.agent_config().clone(),
            task.execution_flags().clone(),
            task.worktree().cloned(),
            cmd.sort_order,
            *task.created_at(),
            completed_at,
            task.has_run(),
        );
        self.task_repo.save(&project_id, &new_task)?;
        Ok(())
    }

    pub fn delete_task(&self, cmd: DeleteTaskCommand) -> Result<(), ApplicationError> {
        let (project_id, path) = self.current_project_id()?;
        let task_id = TaskId::from_string(cmd.id);
        log::info!("[task] Deleting task {}", task_id.0);

        let task = self.task_repo.find_by_id(&project_id, &task_id)?;
        let worktree = task.worktree().cloned();

        if let Err(e) = self.output_repo.clear(&project_id, &task_id) {
            log::warn!("[task] Failed to clear output for {}: {}", task_id.0, e);
        }

        self.task_repo.delete(&project_id, &task_id)?;
        log::info!("[task] Task {} deleted from repository", task_id.0);

        self.event_bus.publish(DomainEvent::Task(TaskDomainEvent::TaskDeleted {
            task_id: task_id.clone(),
            project_id: project_id.clone(),
            worktree: worktree.clone(),
        }));

        // Clean up worktree
        if let Some(wt) = worktree {
            log::info!("[task] Cleaning up worktree {} for task {}", wt.as_str(), task_id.0);
            let project_path = crate::domain::project::value_objects::ProjectPath::new(path);
            if let Err(e) = self.worktree_port.remove(&project_path, &wt) {
                log::error!("[task] Failed to remove worktree for {}: {}", task_id.0, e);
            }
        }

        log::info!("[task] Task {} fully deleted", task_id.0);
        Ok(())
    }

    pub fn move_task(&self, cmd: MoveTaskCommand) -> Result<(), ApplicationError> {
        let (project_id, _) = self.current_project_id()?;
        let task_id = TaskId::from_string(cmd.id);
        let mut task = self.task_repo.find_by_id(&project_id, &task_id)?;
        let column = TaskColumn::from_str(&cmd.column)?;
        let event = task.move_to_column(column, cmd.sort_order)?;
        self.task_repo.save(&project_id, &task)?;
        self.event_bus.publish(DomainEvent::Task(event));
        Ok(())
    }

    pub fn reset_task(&self, cmd: ResetTaskCommand) -> Result<Task, ApplicationError> {
        log::info!("[task] Resetting task: {}", cmd.id);
        let (project_id, path) = self.current_project_id()?;
        let old_task_id = TaskId::from_string(cmd.id);
        let old_task = self.task_repo.find_by_id(&project_id, &old_task_id)?;

        // Clean up worktree
        if let Some(wt) = old_task.worktree() {
            let project_path = crate::domain::project::value_objects::ProjectPath::new(path);
            let _ = self.worktree_port.remove(&project_path, wt);
        }

        let _ = self.output_repo.clear(&project_id, &old_task_id);
        self.task_repo.delete(&project_id, &old_task_id)?;

        // Create replacement with same settings but new ID
        let new_task = Task::create(
            old_task.description().to_string(),
            old_task.agent_config().clone(),
            old_task.execution_flags().clone(),
            old_task.worktree().cloned(),
            old_task.sort_order(),
        );

        // Reconstitute with old column
        let new_task = Task::reconstitute(
            new_task.id().clone(),
            new_task.description().to_string(),
            old_task.column().clone(),
            TaskStatus::Idle,
            new_task.agent_config().clone(),
            new_task.execution_flags().clone(),
            new_task.worktree().cloned(),
            new_task.sort_order(),
            *new_task.created_at(),
            None,
            false,
        );

        self.task_repo.save(&project_id, &new_task)?;

        self.event_bus.publish(DomainEvent::Task(TaskDomainEvent::TaskReset {
            old_task_id,
            new_task_id: new_task.id().clone(),
            project_id,
        }));

        Ok(new_task)
    }

    pub fn mark_task_started(&self, cmd: MarkTaskStartedCommand) -> Result<(), ApplicationError> {
        log::info!("[task] Marking task started: {}", cmd.id);
        let (project_id, _) = self.current_project_id()?;
        let task_id = TaskId::from_string(cmd.id);
        let mut task = self.task_repo.find_by_id(&project_id, &task_id)?;
        let event = task.mark_started()?;
        self.task_repo.save(&project_id, &task)?;
        self.event_bus.publish(DomainEvent::Task(event));
        Ok(())
    }

    pub fn mark_task_completed(
        &self,
        cmd: MarkTaskCompletedCommand,
        project_path: &str,
    ) -> Result<(), ApplicationError> {
        let project_id = ProjectId::from_path(project_path);
        let task_id = TaskId::from_string(cmd.id);
        log::info!("[task] Marking task completed: {} success={}", task_id.0, cmd.success);
        let mut task = self.task_repo.find_by_id(&project_id, &task_id)?;
        let event = task.mark_completed(cmd.success);
        self.task_repo.save(&project_id, &task)?;
        self.event_bus.publish(DomainEvent::Task(event));
        Ok(())
    }

    pub fn mark_task_stopped(&self, cmd: MarkTaskStoppedCommand) -> Result<(), ApplicationError> {
        log::info!("[task] Marking task stopped: {}", cmd.id);
        let (project_id, _) = self.current_project_id()?;
        let task_id = TaskId::from_string(cmd.id);
        let mut task = self.task_repo.find_by_id(&project_id, &task_id)?;
        let event = task.mark_stopped();
        self.task_repo.save(&project_id, &task)?;
        self.event_bus.publish(DomainEvent::Task(event));
        Ok(())
    }

    pub fn clear_output(&self, task_id_str: &str) -> Result<(), ApplicationError> {
        let (project_id, _) = self.current_project_id()?;
        let task_id = TaskId::from_string(task_id_str.to_string());
        let _ = self.output_repo.clear(&project_id, &task_id);
        Ok(())
    }
}
