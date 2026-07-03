//! Agent node — a participant in the orchestration graph.
//!
//! Each agent has a position in capability space as a Grade-1 multivector
//! in G(3,0,0): e1 = reasoning, e2 = tool-use, e3 = generation.

use mp_graph::{ga, GraphError, Mv, NodeId, Positioned};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The kind of work an agent specialises in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Deep reasoning and analysis.
    Reasoner,
    /// Invoking external tools and subprocesses.
    ToolCaller,
    /// Code and content generation.
    CodeGen,
    /// Review, QA, and validation.
    Reviewer,
    /// Routes tasks to other agents.
    Router,
}

/// Runtime status of an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Busy,
    Errored,
    Stopped,
}

/// Per-agent configuration for the LLM backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Path or endpoint for the model this agent uses.
    pub model_endpoint: String,
    /// Optional GBNF grammar file for constrained output.
    pub grammar: Option<String>,
    /// Sampling temperature.
    pub temperature: f32,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model_endpoint: "http://127.0.0.1:8080".into(),
            grammar: None,
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

/// A node in the orchestration graph representing a single agent.
///
/// Its `capabilities` multivector places it in the 3D capability space:
/// - **e1** — reasoning depth
/// - **e2** — tool-use proficiency
/// - **e3** — generation quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentNode {
    pub id: Uuid,
    pub name: String,
    pub agent_type: AgentType,
    #[serde(with = "mp_graph::ga::serde_mv")]
    pub capabilities: Mv,
    pub status: AgentStatus,
    pub config: AgentConfig,
    pub created_at: u64,
}

impl AgentNode {
    /// Create a new agent with the given capability weights.
    ///
    /// `reasoning`, `tool_use`, and `generation` map to e1, e2, e3.
    pub fn new(
        name: impl Into<String>,
        agent_type: AgentType,
        reasoning: f32,
        tool_use: f32,
        generation: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            agent_type,
            capabilities: ga::vector(reasoning, tool_use, generation),
            status: AgentStatus::Idle,
            config: AgentConfig::default(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a new agent with a custom config.
    pub fn with_config(
        name: impl Into<String>,
        agent_type: AgentType,
        reasoning: f32,
        tool_use: f32,
        generation: f32,
        config: AgentConfig,
    ) -> Self {
        let mut agent = Self::new(name, agent_type, reasoning, tool_use, generation);
        agent.config = config;
        agent
    }
}

impl Positioned for AgentNode {
    fn node_id(&self) -> NodeId {
        self.id
    }

    fn position(&self) -> &Mv {
        &self.capabilities
    }

    fn validate_position(&self) -> Result<(), GraphError> {
        if ga::dominant_grade(&self.capabilities) != 1 {
            return Err(GraphError::Invariant(format!(
                "agent {} capabilities is not Grade-1",
                self.id
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_agent_is_grade_one() {
        let agent = AgentNode::new("test", AgentType::Reasoner, 0.9, 0.1, 0.0);
        assert!(agent.validate_position().is_ok());
        assert_eq!(ga::dominant_grade(&agent.capabilities), 1);
    }

    #[test]
    fn new_agent_is_idle() {
        let agent = AgentNode::new("test", AgentType::CodeGen, 0.5, 0.5, 0.8);
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn agent_type_serializes() {
        let json = serde_json::to_string(&AgentType::Router).unwrap();
        assert_eq!(json, "\"Router\"");
    }
}
