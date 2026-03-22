use crate::domain::task::value_objects::TaskId;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SessionDomainEvent {
    SessionStarted {
        task_id: TaskId,
        agent_name: String,
    },
    SessionOutputReceived {
        task_id: TaskId,
        line: String,
        project_path: String,
    },
    SessionEnded {
        task_id: TaskId,
        success: bool,
        project_path: String,
    },
    SessionTerminated {
        task_id: TaskId,
    },
}
