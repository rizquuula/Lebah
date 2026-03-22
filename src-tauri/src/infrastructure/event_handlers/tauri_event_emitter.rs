use tauri::{AppHandle, Emitter};

use crate::application::event_bus::{DomainEvent, EventHandler};
use crate::domain::session::events::SessionDomainEvent;

pub struct TauriEventEmitter {
    app: AppHandle,
}

impl TauriEventEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventHandler for TauriEventEmitter {
    fn handle(&self, event: &DomainEvent) {
        if let DomainEvent::Session(SessionDomainEvent::SessionOutputReceived {
            task_id,
            line,
            ..
        }) = event
        {
            let _ = self
                .app
                .emit(&format!("claude-output-{}", task_id), line);
        }
    }
}
