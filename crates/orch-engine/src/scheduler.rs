//! DAG scheduler — topological execution of pipeline agents.

use std::sync::Arc;

use orch_core::{NodeId, Positioned, Rotored};
use orch_db::OrchDb;

use crate::pipeline::Pipeline;
use crate::task::{TaskResult, TaskState};

/// The scheduler executes a pipeline by walking the DAG in topological order.
pub struct Scheduler {
    db: Arc<OrchDb>,
}

impl Scheduler {
    /// Create a new scheduler backed by the given database.
    pub fn new(db: Arc<OrchDb>) -> Self {
        Self { db }
    }

    /// Execute a pipeline. Finds root agents and processes them.
    pub async fn execute(&self, pipeline: &Pipeline) -> Vec<TaskResult> {
        let mut results = Vec::new();

        // Find root agents (no incoming edges in this pipeline)
        let root_agents: Vec<NodeId> = pipeline
            .graph
            .nodes()
            .filter(|agent| {
                !pipeline
                    .graph
                    .edges()
                    .any(|e| e.endpoints().1 == agent.node_id())
            })
            .map(|a| a.node_id())
            .collect();

        tracing::info!(
            pipeline = %pipeline.name,
            roots = root_agents.len(),
            "Starting pipeline execution"
        );

        for root_id in &root_agents {
            if let Some(agent) = pipeline.graph.node(root_id) {
                tracing::info!(agent = %agent.name, "Executing root agent");
                results.push(TaskResult {
                    agent_id: agent.id,
                    edge_id: uuid::Uuid::nil(),
                    output: format!("Agent '{}' executed (stub)", agent.name),
                    duration_ms: 0,
                    state: TaskState::Complete,
                });
            }
        }

        results
    }

    /// Get a reference to the backing database.
    pub fn db(&self) -> &OrchDb {
        &self.db
    }
}
