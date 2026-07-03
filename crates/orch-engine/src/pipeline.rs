//! Pipeline — a named execution DAG of agents and task edges.

use orch_core::{AgentNode, GeoGraph, Mv, NodeId, TaskEdge};
use uuid::Uuid;

/// A named pipeline: a sub-graph of agents wired together for a specific workflow.
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    pub graph: GeoGraph<AgentNode, TaskEdge>,
}

impl Pipeline {
    /// Create an empty pipeline.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            graph: GeoGraph::new(),
        }
    }

    /// Add an agent to the pipeline.
    pub fn add_agent(&mut self, agent: AgentNode) {
        self.graph.add_node(agent);
    }

    /// Connect two agents with a task edge.
    pub fn connect(
        &mut self,
        from: &AgentNode,
        to: &AgentNode,
        label: impl Into<String>,
    ) -> Result<(), orch_core::GraphError> {
        let edge = TaskEdge::between(from, to, label);
        self.graph.add_edge(edge)?;
        Ok(())
    }

    /// Find the best agent for a task described by the given capability direction.
    ///
    /// Uses GA scoring: the agent whose capability vector has the highest
    /// inner product with the task direction wins.
    pub fn best_agent_for(&self, task_direction: &Mv) -> Option<(NodeId, f32)> {
        self.graph.nearest(&[*task_direction], 1).into_iter().next()
    }

    /// Return all agents qualified for a task (score above threshold).
    pub fn qualified_agents(&self, task_direction: &Mv, threshold: f32) -> Vec<NodeId> {
        self.graph.project_above(&[*task_direction], threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orch_core::{AgentNode, AgentType, ga};

    #[test]
    fn best_agent_selects_highest_scorer() {
        let mut pipeline = Pipeline::new("test");
        let reasoner = AgentNode::new("reasoner", AgentType::Reasoner, 0.9, 0.1, 0.0);
        let coder = AgentNode::new("coder", AgentType::CodeGen, 0.1, 0.2, 0.9);
        let r_id = reasoner.id;
        pipeline.add_agent(reasoner);
        pipeline.add_agent(coder);

        let reasoning_task = ga::vector(1.0, 0.0, 0.0);
        let (best_id, _) = pipeline.best_agent_for(&reasoning_task).unwrap();
        assert_eq!(best_id, r_id);
    }

    #[test]
    fn qualified_agents_filters_by_threshold() {
        let mut pipeline = Pipeline::new("test");
        let r = AgentNode::new("reasoner", AgentType::Reasoner, 0.9, 0.1, 0.0);
        let c = AgentNode::new("coder", AgentType::CodeGen, 0.0, 0.1, 0.9);
        let r_id = r.id;
        pipeline.add_agent(r);
        pipeline.add_agent(c);

        let reasoning_task = ga::vector(1.0, 0.0, 0.0);
        let qualified = pipeline.qualified_agents(&reasoning_task, 0.15);
        assert_eq!(qualified.len(), 1);
        assert_eq!(qualified[0], r_id);
    }
}
