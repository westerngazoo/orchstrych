//! Error types for orch-core.

use mp_graph::GraphError;

/// Core orchestration errors.
#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),

    #[error("Agent not found: {0}")]
    AgentNotFound(uuid::Uuid),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
