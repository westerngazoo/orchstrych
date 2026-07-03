//! Inference client trait and backend implementations.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use crate::error::LlmError;
use crate::grammar::Grammar;

/// Trait for local LLM inference backends.
#[async_trait]
pub trait InferenceClient: Send + Sync {
    /// Run inference with an optional grammar constraint.
    async fn infer(&self, prompt: &str, grammar: Option<&Grammar>) -> Result<String, LlmError>;

    /// Run inference and deserialize the result into a typed struct.
    async fn infer_structured<T: DeserializeOwned + Send>(
        &self,
        prompt: &str,
        grammar: &Grammar,
    ) -> Result<T, LlmError> {
        let raw = self.infer(prompt, Some(grammar)).await?;
        serde_json::from_str(&raw).map_err(|e| LlmError::Parse(format!("{e}: {raw}")))
    }
}

// --- llama.cpp backend ---

/// llama.cpp HTTP server client with GBNF grammar support.
pub struct LlamaCppClient {
    endpoint: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct LlamaCppRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    grammar: Option<String>,
    temperature: f32,
    n_predict: u32,
}

#[derive(Deserialize)]
struct LlamaCppResponse {
    content: String,
}

impl LlamaCppClient {
    /// Create a new llama.cpp client pointing at the given server endpoint.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl InferenceClient for LlamaCppClient {
    async fn infer(&self, prompt: &str, grammar: Option<&Grammar>) -> Result<String, LlmError> {
        let req = LlamaCppRequest {
            prompt: prompt.to_string(),
            grammar: grammar.map(|g| g.content.clone()),
            temperature: 0.7,
            n_predict: 2048,
        };
        let resp = self
            .client
            .post(format!("{}/completion", self.endpoint))
            .json(&req)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::Inference(format!("{status}: {body}")));
        }
        let parsed: LlamaCppResponse = resp.json().await?;
        Ok(parsed.content)
    }
}

// --- Ollama backend ---

/// Ollama HTTP API client.
pub struct OllamaClient {
    endpoint: String,
    model: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaClient {
    /// Create a new Ollama client.
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl InferenceClient for OllamaClient {
    async fn infer(&self, prompt: &str, grammar: Option<&Grammar>) -> Result<String, LlmError> {
        let req = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            format: grammar.map(|_| "json".to_string()),
        };
        let resp = self
            .client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&req)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::Inference(format!("{status}: {body}")));
        }
        let parsed: OllamaResponse = resp.json().await?;
        Ok(parsed.response)
    }
}
