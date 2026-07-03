//! orch-db — redb-backed persistence for the orchestration graph.
//!
//! Stores agent nodes and task edges in separate redb tables, using bincode
//! serialization. Supports MVCC reads so background agents can analyze the
//! graph topology without blocking the scheduler's write path.

mod store;

pub use store::OrchDb;
