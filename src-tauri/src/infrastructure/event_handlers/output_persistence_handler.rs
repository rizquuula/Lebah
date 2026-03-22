use std::sync::Arc;

use crate::application::event_bus::{DomainEvent, EventHandler};
use crate::domain::project::value_objects::ProjectId;
use crate::domain::repositories::OutputRepository;
use crate::domain::session::events::SessionDomainEvent;

pub struct OutputPersistenceHandler {
    output_repo: Arc<dyn OutputRepository>,
    output_buffers: Arc<std::sync::Mutex<std::collections::HashMap<String, Vec<String>>>>,
}

impl OutputPersistenceHandler {
    pub fn new(
        output_repo: Arc<dyn OutputRepository>,
        output_buffers: Arc<std::sync::Mutex<std::collections::HashMap<String, Vec<String>>>>,
    ) -> Self {
        Self { output_repo, output_buffers }
    }
}

impl EventHandler for OutputPersistenceHandler {
    fn handle(&self, event: &DomainEvent) {
        if let DomainEvent::Session(SessionDomainEvent::SessionOutputReceived {
            task_id,
            line,
            project_path,
        }) = event
        {
            // Update in-memory buffer
            if let Ok(mut buffers) = self.output_buffers.lock() {
                buffers.entry(task_id.0.clone()).or_default().push(line.clone());
            } else {
                log::error!("[output_handler] Failed to lock output buffer for task {}", task_id.0);
            }
            // Persist to file
            if !project_path.is_empty() {
                let project_id = ProjectId::from_path(project_path);
                if let Err(e) = self.output_repo.append(&project_id, task_id, line) {
                    log::error!("[output_handler] Failed to persist output for task {}: {}", task_id.0, e);
                }
            }
        }
    }
}
