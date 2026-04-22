//! Provider abstraction for multi-backend LLM support.
//!
//! Routes requests to the correct backend (Ollama, OpenAI, Anthropic)
//! based on user configuration. Each provider implements the same streaming
//! interface but translates messages to its own API format.

pub mod openai;
pub mod anthropic;

use serde::{Deserialize, Serialize};

/// Supported LLM providers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Ollama,
    OpenAI,
    Anthropic,
}

/// Runtime configuration for the active provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Which provider to use for requests.
    pub provider: Provider,
    /// Model name to send to the provider.
    pub model: String,
    /// Base URL for the provider API (Ollama: http://127.0.0.1:11434, OpenAI: https://api.openai.com/v1, Anthropic: https://api.anthropic.com).
    pub base_url: String,
    /// API key for cloud providers (empty for Ollama).
    pub api_key: String,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Ollama,
            model: "gemini-3-flash-preview".to_string(),
            base_url: "http://127.0.0.1:11434".to_string(),
            api_key: String::new(),
        }
    }
}

/// A tool call from a model response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for the tool call (used to match results).
    pub id: String,
    /// The function/tool name (e.g., "computer_click").
    pub name: String,
    /// JSON string of arguments.
    pub arguments: String,
}

/// A chunk of streaming response from any provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ProviderChunk {
    /// A text token from the model.
    Token(String),
    /// A thinking/reasoning token.
    ThinkingToken(String),
    /// The model is requesting one or more tool calls.
    ToolCalls(Vec<ToolCall>),
    /// Streaming is complete.
    Done,
    /// Streaming was cancelled by the user.
    Cancelled,
    /// An error occurred.
    Error(String),
}

/// Returns default base URLs for each provider.
pub fn default_base_url(provider: &Provider) -> &'static str {
    match provider {
        Provider::Ollama => "http://127.0.0.1:11434",
        Provider::OpenAI => "https://api.openai.com/v1",
        Provider::Anthropic => "https://api.anthropic.com",
    }
}

/// Returns recommended models for each provider.
pub fn default_models(provider: &Provider) -> &'static [&'static str] {
    match provider {
        Provider::Ollama => &["gemini-3-flash-preview", "llama3.2-vision", "llama3.2", "mistral"],
        Provider::OpenAI => &["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"],
        Provider::Anthropic => &["claude-sonnet-4-20250514", "claude-3-5-sonnet-20241022", "claude-3-5-haiku-20241022"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_serialization() {
        assert_eq!(serde_json::to_string(&Provider::Ollama).unwrap(), "\"ollama\"");
        assert_eq!(serde_json::to_string(&Provider::OpenAI).unwrap(), "\"openai\"");
        assert_eq!(serde_json::to_string(&Provider::Anthropic).unwrap(), "\"anthropic\"");
    }

    #[test]
    fn provider_config_default() {
        let config = ProviderConfig::default();
        assert_eq!(config.provider, Provider::Ollama);
        assert_eq!(config.base_url, "http://127.0.0.1:11434");
        assert!(config.api_key.is_empty());
    }

    #[test]
    fn default_base_urls() {
        assert_eq!(default_base_url(&Provider::Ollama), "http://127.0.0.1:11434");
        assert_eq!(default_base_url(&Provider::OpenAI), "https://api.openai.com/v1");
        assert_eq!(default_base_url(&Provider::Anthropic), "https://api.anthropic.com");
    }

    #[test]
    fn default_models_not_empty() {
        for provider in &[Provider::Ollama, Provider::OpenAI, Provider::Anthropic] {
            assert!(!default_models(provider).is_empty());
        }
    }
}