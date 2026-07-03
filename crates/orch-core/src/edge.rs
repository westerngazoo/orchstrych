//! Task edge — a directed connection between two agents in the orchestration graph.
//!
//! Each edge carries an even-grade rotor that defines how context transforms
//! as it flows from the source agent to the target agent.

use mp_graph::{ga, GraphError, Mv, NodeId, Positioned, Rotored};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::AgentNode;

/// Runtime status of a task edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeStatus {
    Pending,
    Active,
    Complete,
    Failed,
}

/// A directed edge in the orchestration graph representing a task handoff.
///
/// The `rotor` is an even-grade multivector computed from the geometric product
/// of the source and target agent positions. It defines the transformation
/// applied to context as it flows along this edge via the sandwich product
/// `R · v · ~R`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEdge {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
    pub task_label: String,
    #[serde(with = "mp_graph::ga::serde_mv")]
    pub rotor: Mv,
    pub priority: f32,
    pub status: EdgeStatus,
}

impl TaskEdge {
    /// Create a new task edge between two agents.
    ///
    /// The rotor is computed as the normalized even-grade part of the geometric
    /// product of the two agents' capability vectors.
    pub fn between(from: &AgentNode, to: &AgentNode, label: impl Into<String>) -> Self {
        let gp = *from.position() * *to.position();
        let rotor = ga::normalize(&ga::even_grade(&gp));
        Self {
            id: Uuid::new_v4(),
            from: from.id,
            to: to.id,
            task_label: label.into(),
            rotor,
            priority: 1.0,
            status: EdgeStatus::Pending,
        }
    }

    /// Create a task edge with a custom priority.
    pub fn with_priority(mut self, priority: f32) -> Self {
        self.priority = priority;
        self
    }
}

impl Rotored for TaskEdge {
    fn edge_id(&self) -> mp_graph::EdgeId {
        self.id
    }

    fn endpoints(&self) -> (NodeId, NodeId) {
        (self.from, self.to)
    }

    fn rotor(&self) -> &Mv {
        &self.rotor
    }

    fn validate_rotor(&self) -> Result<(), GraphError> {
        if !ga::is_even_grade(&self.rotor) {
            return Err(GraphError::Invariant(format!(
                "edge {} rotor is not even-grade",
                self.id
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{AgentNode, AgentType};

    #[test]
    fn edge_rotor_is_even_grade() {
        let a = AgentNode::new("reasoner", AgentType::Reasoner, 0.9, 0.1, 0.0);
        let b = AgentNode::new("coder", AgentType::CodeGen, 0.2, 0.3, 0.9);
        let edge = TaskEdge::between(&a, &b, "analyze then generate");
        assert!(edge.validate_rotor().is_ok());
        assert!(ga::is_even_grade(&edge.rotor));
    }

    #[test]
    fn edge_default_status_is_pending() {
        let a = AgentNode::new("a", AgentType::Router, 0.5, 0.5, 0.5);
        let b = AgentNode::new("b", AgentType::ToolCaller, 0.1, 0.9, 0.1);
        let edge = TaskEdge::between(&a, &b, "route");
        assert_eq!(edge.status, EdgeStatus::Pending);
    }

    #[test]
    fn with_priority_sets_priority() {
        let a = AgentNode::new("a", AgentType::Reasoner, 0.8, 0.1, 0.1);
        let b = AgentNode::new("b", AgentType::Reviewer, 0.7, 0.2, 0.3);
        let edge = TaskEdge::between(&a, &b, "review").with_priority(5.0);
        assert_eq!(edge.priority, 5.0);
    }
}
