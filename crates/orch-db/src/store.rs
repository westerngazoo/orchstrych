//! redb-backed store for the orchestration graph.

use std::path::Path;

use orch_core::{AgentNode, GeoGraph, GraphError, Positioned, Rotored, TaskEdge};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::Serialize;

const AGENTS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("agents");
const TASK_EDGES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("task_edges");
const TASK_LOG: TableDefinition<u64, &[u8]> = TableDefinition::new("task_log");

/// Persistence layer for the orchestration graph.
pub struct OrchDb {
    db: Database,
}

impl OrchDb {
    /// Create or open the database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, GraphError> {
        let db = Database::create(path).map_err(db_err)?;
        Ok(Self { db })
    }

    /// Persist the full agent graph in one write transaction.
    pub fn save_graph(&self, graph: &GeoGraph<AgentNode, TaskEdge>) -> Result<(), GraphError> {
        let txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut agents = txn.open_table(AGENTS).map_err(db_err)?;
            for n in graph.nodes() {
                let value = bincode::serialize(n).map_err(ser_err)?;
                agents
                    .insert(n.node_id().as_bytes().as_slice(), value.as_slice())
                    .map_err(db_err)?;
            }
            let mut edges = txn.open_table(TASK_EDGES).map_err(db_err)?;
            for e in graph.edges() {
                let value = bincode::serialize(e).map_err(ser_err)?;
                edges
                    .insert(e.edge_id().as_bytes().as_slice(), value.as_slice())
                    .map_err(db_err)?;
            }
        }
        txn.commit().map_err(db_err)?;
        tracing::info!(
            "Graph saved: {} agents, {} edges",
            graph.node_count(),
            graph.edge_count()
        );
        Ok(())
    }

    /// Load the full agent graph from disk, re-validating GA invariants.
    pub fn load_graph(&self) -> Result<GeoGraph<AgentNode, TaskEdge>, GraphError> {
        let mut graph = GeoGraph::new();
        let txn = self.db.begin_read().map_err(db_err)?;

        let agents = txn.open_table(AGENTS).map_err(db_err)?;
        for entry in agents.iter().map_err(db_err)? {
            let (_, value) = entry.map_err(db_err)?;
            let node: AgentNode = bincode::deserialize(value.value()).map_err(ser_err)?;
            node.validate_position()?;
            graph.add_node(node);
        }

        let edges = txn.open_table(TASK_EDGES).map_err(db_err)?;
        for entry in edges.iter().map_err(db_err)? {
            let (_, value) = entry.map_err(db_err)?;
            let edge: TaskEdge = bincode::deserialize(value.value()).map_err(ser_err)?;
            edge.validate_rotor()?;
            graph.add_edge(edge)?;
        }

        tracing::info!(
            "Graph loaded: {} agents, {} edges",
            graph.node_count(),
            graph.edge_count()
        );
        Ok(graph)
    }

    /// Append an entry to the execution log.
    pub fn log_task(&self, timestamp: u64, entry: &impl Serialize) -> Result<(), GraphError> {
        let txn = self.db.begin_write().map_err(db_err)?;
        {
            let mut log = txn.open_table(TASK_LOG).map_err(db_err)?;
            let value = bincode::serialize(entry).map_err(ser_err)?;
            log.insert(timestamp, value.as_slice()).map_err(db_err)?;
        }
        txn.commit().map_err(db_err)?;
        Ok(())
    }
}

fn db_err<E: std::fmt::Display>(e: E) -> GraphError {
    GraphError::Db(e.to_string())
}

fn ser_err(e: bincode::Error) -> GraphError {
    GraphError::Db(format!("bincode: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use orch_core::{AgentNode, AgentType, TaskEdge};

    #[test]
    fn round_trip_graph() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test.redb");

        let a = AgentNode::new("reasoner", AgentType::Reasoner, 0.9, 0.1, 0.0);
        let b = AgentNode::new("coder", AgentType::CodeGen, 0.2, 0.3, 0.9);
        let edge = TaskEdge::between(&a, &b, "reason then code");
        let (a_id, b_id) = (a.id, b.id);

        let mut graph = GeoGraph::new();
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(edge).expect("endpoints exist");

        let db = OrchDb::open(&path).expect("open db");
        db.save_graph(&graph).expect("save");

        let loaded = db.load_graph().expect("load");
        assert_eq!(loaded.node_count(), 2);
        assert_eq!(loaded.edge_count(), 1);
        assert!(loaded.node(&a_id).is_some());
        assert!(loaded.node(&b_id).is_some());
    }
}
