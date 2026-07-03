/// Tool execution errors.
#[derive(thiserror::Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Tool output is not valid UTF-8")]
    InvalidOutput,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
