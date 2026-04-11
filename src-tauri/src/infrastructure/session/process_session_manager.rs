use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::application::ports::SessionManagerPort;
use crate::domain::task::value_objects::TaskId;

pub struct ProcessSessionManager {
    output_buffers: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl ProcessSessionManager {
    pub fn new(output_buffers: Arc<Mutex<HashMap<String, Vec<String>>>>) -> Self {
        Self { output_buffers }
    }
}

impl SessionManagerPort for ProcessSessionManager {
    fn get_live_buffer(&self, task_id: &TaskId) -> Vec<String> {
        self.output_buffers
            .lock()
            .ok()
            .and_then(|b| b.get(&task_id.0).cloned())
            .unwrap_or_default()
    }
}
