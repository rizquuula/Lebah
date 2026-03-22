use std::sync::{Arc, Mutex};

use crate::domain::session::events::SessionDomainEvent;
use crate::domain::task::events::TaskDomainEvent;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DomainEvent {
    Task(TaskDomainEvent),
    Session(SessionDomainEvent),
}

pub trait EventHandler: Send + Sync + 'static {
    fn handle(&self, event: &DomainEvent);
}

pub trait DomainEventBus: Send + Sync + 'static {
    fn publish(&self, event: DomainEvent);
    fn subscribe(&self, handler: Arc<dyn EventHandler>);
}

pub struct InProcessEventBus {
    handlers: Mutex<Vec<Arc<dyn EventHandler>>>,
}

impl InProcessEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(Vec::new()),
        }
    }
}

impl DomainEventBus for InProcessEventBus {
    fn publish(&self, event: DomainEvent) {
        let handlers = self.handlers.lock().unwrap();
        for handler in handlers.iter() {
            handler.handle(&event);
        }
    }

    fn subscribe(&self, handler: Arc<dyn EventHandler>) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push(handler);
    }
}
