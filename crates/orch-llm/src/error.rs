/// LLM inference errors.
#[derive(thiserror::Error, Debug)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Failed to parse LLM response: {0}")]
    Parse(String),

    #[error("Grammar file not found: {0}")]
    GrammarNotFound(String),

    #[error("Inference failed: {0}")]
    Inference(String),
}

impl From<reqwest::Error> for LlmError {
    fn from(e: reqwest::Error) -> Self {
        LlmError::Http(e.to_string())
    }
}
