//! orch-tools — native subprocess tool runner for the orchestration engine.
//!
//! Delegates string processing to native binaries (awk, grep, sed, jq) instead
//! of burning LLM attention on parsing. Native parsing is O(N) vs the LLM's
//! O(N² · d) attention complexity.

pub mod error;
pub mod runner;

pub use error::ToolError;
pub use runner::ToolRunner;
