//! orch-llm — local LLM inference client with grammar-constrained decoding.

pub mod client;
pub mod error;
pub mod grammar;

pub use client::{InferenceClient, LlamaCppClient, OllamaClient};
pub use error::LlmError;
pub use grammar::Grammar;
