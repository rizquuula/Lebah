use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::agent::runner::AgentRunner;

/// Registry of available agent runners.
/// New agents are added by registering an impl of AgentRunner — no existing code is modified.
pub struct AgentRegistry {
    runners: HashMap<String, Arc<dyn AgentRunner>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            runners: HashMap::new(),
        }
    }

    pub fn register(&mut self, runner: Arc<dyn AgentRunner>) {
        self.runners.insert(runner.name().to_string(), runner);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn AgentRunner>> {
        self.runners.get(name).cloned()
    }

    pub fn default_runner(&self) -> Option<Arc<dyn AgentRunner>> {
        self.runners.values().next().cloned()
    }
}
