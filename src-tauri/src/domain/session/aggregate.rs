use std::time::Instant;

use crate::domain::task::value_objects::TaskId;

#[allow(dead_code)]
#[derive(Debug)]
pub enum SessionState {
    Idle,
    Running { started_at: Instant },
    Completed { success: bool },
    Failed { reason: String },
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Session {
    task_id: TaskId,
    state: SessionState,
    agent_name: String,
}

#[allow(dead_code)]
impl Session {
    pub fn new(task_id: TaskId, agent_name: String) -> Self {
        Self {
            task_id,
            state: SessionState::Idle,
            agent_name,
        }
    }

    pub fn start(&mut self) {
        self.state = SessionState::Running {
            started_at: Instant::now(),
        };
    }

    pub fn complete(&mut self, success: bool) {
        self.state = SessionState::Completed { success };
    }

    pub fn fail(&mut self, reason: String) {
        self.state = SessionState::Failed { reason };
    }

    pub fn task_id(&self) -> &TaskId { &self.task_id }
    pub fn state(&self) -> &SessionState { &self.state }
    pub fn agent_name(&self) -> &str { &self.agent_name }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::value_objects::TaskId;

    fn make_session() -> Session {
        Session::new(TaskId::new(), "claude".to_string())
    }

    #[test]
    fn new_session_is_idle() {
        let s = make_session();
        assert!(matches!(s.state(), SessionState::Idle));
        assert_eq!(s.agent_name(), "claude");
    }

    #[test]
    fn start_transitions_to_running() {
        let mut s = make_session();
        s.start();
        assert!(matches!(s.state(), SessionState::Running { .. }));
    }

    #[test]
    fn complete_success() {
        let mut s = make_session();
        s.start();
        s.complete(true);
        assert!(matches!(s.state(), SessionState::Completed { success: true }));
    }

    #[test]
    fn fail_with_reason() {
        let mut s = make_session();
        s.start();
        s.fail("conflict".to_string());
        assert!(matches!(s.state(), SessionState::Failed { reason } if reason == "conflict"));
    }
}
