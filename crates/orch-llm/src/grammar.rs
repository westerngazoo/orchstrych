//! GBNF grammar support for constrained LLM decoding.

use std::path::Path;
use crate::error::LlmError;

/// A GBNF grammar that constrains the LLM's output to a specific format.
#[derive(Debug, Clone)]
pub struct Grammar {
    /// The raw GBNF grammar string.
    pub content: String,
    /// Optional name for logging.
    pub name: Option<String>,
}

impl Grammar {
    /// Create a grammar from a raw GBNF string.
    pub fn from_string(content: impl Into<String>) -> Self {
        Self { content: content.into(), name: None }
    }

    /// Load a grammar from a file.
    pub fn from_file(path: &Path) -> Result<Self, LlmError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| LlmError::GrammarNotFound(format!("{}: {e}", path.display())))?;
        let name = path.file_stem().and_then(|s| s.to_str()).map(String::from);
        Ok(Self { content, name })
    }

    /// Built-in grammar: forces JSON tool-call output.
    pub fn tool_call() -> Self {
        Self {
            content: r#"root   ::= "{" ws "\"tool\"" ws ":" ws string "," ws "\"args\"" ws ":" ws object "}" ws
string ::= "\"" [a-zA-Z0-9_-]+ "\""
object ::= "{" ws (pair ("," ws pair)*)? ws "}"
pair   ::= string ws ":" ws value
value  ::= string | number | object | "true" | "false" | "null"
number ::= "-"? [0-9]+ ("." [0-9]+)?
ws     ::= [ \t\n]*"#.into(),
            name: Some("tool_call".into()),
        }
    }

    /// Built-in grammar: forces routing decision output.
    pub fn routing_decision() -> Self {
        Self {
            content: r#"root   ::= "{" ws "\"next_agent\"" ws ":" ws string "," ws "\"context\"" ws ":" ws string "}" ws
string ::= "\"" [^"]* "\""
ws     ::= [ \t\n]*"#.into(),
            name: Some("routing_decision".into()),
        }
    }
}
