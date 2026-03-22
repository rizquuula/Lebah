use std::sync::Arc;

use crate::application::event_bus::{DomainEvent, EventHandler};
use crate::application::task::commands::MarkTaskCompletedCommand;
use crate::application::task::service::TaskApplicationService;
use crate::domain::session::events::SessionDomainEvent;

pub struct SessionStatusHandler {
    task_service: Arc<TaskApplicationService>,
}

impl SessionStatusHandler {
    pub fn new(task_service: Arc<TaskApplicationService>) -> Self {
        Self { task_service }
    }
}

impl EventHandler for SessionStatusHandler {
    fn handle(&self, event: &DomainEvent) {
        if let DomainEvent::Session(SessionDomainEvent::SessionEnded {
            task_id,
            success,
            project_path,
        }) = event
        {
            eprintln!(
                "[session_status_handler] task={} success={} project={}",
                task_id, success, project_path
            );
            let _ = self.task_service.mark_task_completed(
                MarkTaskCompletedCommand {
                    id: task_id.0.clone(),
                    success: *success,
                },
                project_path,
            );
        }
    }
}
