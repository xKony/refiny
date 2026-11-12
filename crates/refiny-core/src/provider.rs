//! OpenAI-compatible chat-completions client.
//!
//! One client covers OpenAI, OpenRouter, Ollama, LM Studio, vLLM, and anything
//! else that exposes `POST {base_url}/chat/completions`.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::Config;

/// A text transformation provider.
pub trait Polisher {
    fn polish(&self, system_prompt: &str, text: &str) -> Result<String, ProviderError>;
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("provider returned HTTP {status}: {body}")]
    Status { status: u16, body: String },
    #[error("provider returned an error: {0}")]
    Api(String),
    #[error("provider returned no choices")]
    NoChoices,
    #[error("provider returned empty text")]
    Empty,
    #[error("could not decode provider response: {0}")]
    Decode(#[from] serde_json::Error),
}

/// Calls any endpoint exposing the `/chat/completions` shape.
pub struct OpenAiCompatible {
    base_url: String,
    model: String,
    api_key: Option<String>,
    client: reqwest::blocking::Client,
}

impl OpenAiCompatible {
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key: Option<String>,
        timeout: Duration,
    ) -> Result<Self, ProviderError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()?;
        Ok(Self {
            base_url: base_url.into(),
            model: model.into(),
            api_key,
            client,
        })
    }

    pub fn from_config(config: &Config) -> Result<Self, ProviderError> {
        Self::new(
            config.provider.base_url.clone(),
            config.provider.model.clone(),
            config.api_key(),
            Duration::from_millis(config.timeout_ms),
        )
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: [ChatMessage; 2],
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

impl Polisher for OpenAiCompatible {
    fn polish(&self, system_prompt: &str, text: &str) -> Result<String, ProviderError> {
        let body = ChatRequest {
            model: self.model.clone(),
            messages: [
                ChatMessage {
                    role: "system".into(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: text.to_string(),
                },
            ],
            temperature: 0.0,
        };

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut request = self.client.post(url).json(&body);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send()?;
        let status = response.status();
        let payload = response.text()?;

        if !status.is_success() {
            return Err(ProviderError::Status {
                status: status.as_u16(),
                body: truncate(&payload, 300),
            });
        }

        let parsed: ChatResponse = serde_json::from_str(&payload)?;
        if let Some(error) = parsed.error {
            return Err(ProviderError::Api(error.message));
        }

        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or(ProviderError::NoChoices)?;
        let text = choice.message.content.trim().to_string();
        if text.is_empty() {
            return Err(ProviderError::Empty);
        }
        Ok(text)
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[test]
    fn calls_chat_completions_and_returns_text() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/chat/completions")
                .header("authorization", "Bearer test-key");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{"choices":[{"message":{"role":"assistant","content":"Polished text."}}]}"#,
                );
        });

        let client = OpenAiCompatible::new(
            server.base_url(),
            "small-model",
            Some("test-key".into()),
            Duration::from_secs(5),
        )
        .unwrap();

        let result = client.polish("system", "rough text").unwrap();

        assert_eq!(result, "Polished text.");
        mock.assert();
    }

    #[test]
    fn surfaces_provider_errors() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/chat/completions");
            then.status(401)
                .header("content-type", "application/json")
                .body(r#"{"error":{"message":"invalid api key"}}"#);
        });

        let client = OpenAiCompatible::new(
            server.base_url(),
            "small-model",
            Some("bad".into()),
            Duration::from_secs(5),
        )
        .unwrap();

        let error = client.polish("system", "text").unwrap_err();

        assert!(error.to_string().contains("401"));
        mock.assert();
    }
}
