//! orch-core — agent and task types for the orchestration graph.
//!
//! Defines [`AgentNode`] (implementing [`Positioned`]) and [`TaskEdge`]
//! (implementing [`Rotored`]) for use with [`GeoGraph`]. Re-exports the
//! essential mp-graph types so downstream crates need only depend on orch-core.

pub mod agent;
pub mod edge;
pub mod error;

pub use agent::{AgentConfig, AgentNode, AgentStatus, AgentType};
pub use edge::{EdgeStatus, TaskEdge};
pub use error::CoreError;

pub use mp_graph::{ga, EdgeId, GeoGraph, GraphDb, GraphError, Mv, NodeId, Positioned, Rotored};
