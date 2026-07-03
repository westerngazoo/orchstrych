//! Task state machine for pipeline execution.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// State of a task in the execution pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Running,
    Complete,
    Failed,
    Exhausted,
}

/// Result of a completed task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub agent_id: Uuid,
    pub edge_id: Uuid,
    pub output: String,
    pub duration_ms: u64,
    pub state: TaskState,
}
