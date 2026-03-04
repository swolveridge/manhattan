use std::env;

use reqwest::StatusCode;
use serde_json::Value;
use url::Url;

use crate::llm::error::InvocationError;
use crate::llm::protocol::{ChatRequestBody, ChatResponseBody};
use crate::llm::types::{ChatRequest, ChatResponse};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1/";

#[derive(Debug, Clone, Default)]
pub struct ClientConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

pub struct OpenAiCompatibleClient {
    http: reqwest::Client,
    api_key: String,
    base_url: Url,
}

impl OpenAiCompatibleClient {
    pub fn new(api_key: String) -> Result<Self, InvocationError> {
        Self::from_config(ClientConfig {
            api_key: Some(api_key),
            base_url: None,
        })
    }

    pub fn from_config(config: ClientConfig) -> Result<Self, InvocationError> {
        let api_key = config
            .api_key
            .or_else(|| env::var("OPENAI_API_KEY").ok())
            .ok_or(InvocationError::MissingApiKey)?;

        let base_url_raw = config
            .base_url
            .or_else(|| env::var("OPENAI_BASE_URL").ok())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let base_url =
            Url::parse(&base_url_raw).map_err(|err| InvocationError::InvalidBaseUrl {
                value: base_url_raw.clone(),
                message: err.to_string(),
            })?;

        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
            base_url,
        })
    }

    pub fn with_base_url(mut self, base_url: String) -> Result<Self, InvocationError> {
        self.base_url = Url::parse(&base_url).map_err(|err| InvocationError::InvalidBaseUrl {
            value: base_url,
            message: err.to_string(),
        })?;
        Ok(self)
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, InvocationError> {
        let url = self.base_url.join("chat/completions").map_err(|err| {
            InvocationError::InvalidBaseUrl {
                value: self.base_url.to_string(),
                message: err.to_string(),
            }
        })?;

        let response = self
            .http
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&ChatRequestBody::from(request))
            .send()
            .await
            .map_err(|err| InvocationError::Network(err.to_string()))?;

        self.parse_response(response.status(), response.text().await)
            .await
    }

    async fn parse_response(
        &self,
        status: StatusCode,
        body_result: Result<String, reqwest::Error>,
    ) -> Result<ChatResponse, InvocationError> {
        let body = body_result.map_err(|err| InvocationError::Network(err.to_string()))?;

        if !status.is_success() {
            return Err(InvocationError::Http {
                status: status.as_u16(),
                body,
            });
        }

        let value: Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(err) if err.is_eof() => {
                return Err(InvocationError::Parse(format!(
                    "truncated JSON response: {body}"
                )));
            }
            Err(err) => {
                return Err(InvocationError::Parse(format!(
                    "invalid JSON response ({err}): {body}"
                )));
            }
        };

        if let Some(error_payload) = value.get("error") {
            return Err(InvocationError::Provider(error_payload.to_string()));
        }

        let first_choice = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .ok_or_else(|| InvocationError::InvalidResponse("missing choices".to_string()))?;

        let finish_reason = first_choice.get("finish_reason").and_then(Value::as_str);
        if finish_reason == Some("error") {
            return Err(InvocationError::Provider(first_choice.to_string()));
        }

        if let Some(choice_error) = first_choice.get("error").or_else(|| {
            first_choice
                .get("message")
                .and_then(|message| message.get("error"))
        }) {
            return Err(InvocationError::Provider(choice_error.to_string()));
        }

        let has_content = first_choice
            .get("message")
            .and_then(|m| m.get("content"))
            .is_some_and(|v| !v.is_null() && !v.as_str().is_some_and(|s| s.trim().is_empty()));
        let has_tool_calls = first_choice
            .get("message")
            .and_then(|m| m.get("tool_calls"))
            .is_some_and(|v| v.is_array() && !v.as_array().unwrap_or(&Vec::new()).is_empty());

        if !has_content && !has_tool_calls {
            return Err(InvocationError::InvalidResponse(
                "assistant returned no content and no tool calls".to_string(),
            ));
        }

        let parsed = serde_json::from_value::<ChatResponseBody>(value).map_err(|err| {
            InvocationError::Parse(format!("failed to decode chat response ({err}): {body}"))
        })?;

        ChatResponse::try_from(parsed)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::llm::types::{Message, Role};

    fn request() -> ChatRequest {
        ChatRequest {
            model: "openai/gpt-5".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: Some("hello".to_string()),
                tool_call_id: None,
                tool_calls: None,
            }],
            response_format: None,
            tools: None,
            tool_choice: None,
            temperature: None,
            max_tokens: None,
            reasoning_effort: None,
        }
    }

    #[tokio::test]
    async fn returns_successful_response() {
        let server = MockServer::start().await;
        let payload = json!({
            "id": "chatcmpl-1",
            "model": "openai/gpt-5",
            "choices": [{
                "index": 0,
                "finish_reason": "stop",
                "message": {
                    "role": "assistant",
                    "content": "hi"
                }
            }],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(payload))
            .mount(&server)
            .await;

        let client = OpenAiCompatibleClient::new("test-key".to_string())
            .unwrap()
            .with_base_url(server.uri())
            .unwrap();

        let response = client.chat(request()).await.expect("success");
        assert_eq!(response.id, "chatcmpl-1");
        assert_eq!(response.choices[0].message.content.as_deref(), Some("hi"));
    }

    #[tokio::test]
    async fn returns_http_error_on_non_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({"error": "bad key"})))
            .mount(&server)
            .await;

        let client = OpenAiCompatibleClient::new("test-key".to_string())
            .unwrap()
            .with_base_url(server.uri())
            .unwrap();

        let err = client.chat(request()).await.expect_err("expected error");
        assert!(matches!(err, InvocationError::Http { status: 401, .. }));
    }

    #[tokio::test]
    async fn returns_provider_error_from_200_payload() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "error": {"message": "rate limited"}
            })))
            .mount(&server)
            .await;

        let client = OpenAiCompatibleClient::new("test-key".to_string())
            .unwrap()
            .with_base_url(server.uri())
            .unwrap();

        let err = client.chat(request()).await.expect_err("expected error");
        assert!(matches!(err, InvocationError::Provider(_)));
    }

    #[tokio::test]
    async fn rejects_empty_actionless_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-1",
                "model": "openai/gpt-5",
                "choices": [{
                    "index": 0,
                    "finish_reason": "stop",
                    "message": {
                        "role": "assistant",
                        "content": null
                    }
                }]
            })))
            .mount(&server)
            .await;

        let client = OpenAiCompatibleClient::new("test-key".to_string())
            .unwrap()
            .with_base_url(server.uri())
            .unwrap();

        let err = client.chat(request()).await.expect_err("expected error");
        assert!(matches!(err, InvocationError::InvalidResponse(_)));
    }

    #[tokio::test]
    async fn supports_tool_call_without_content() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-1",
                "model": "openai/gpt-5",
                "choices": [{
                    "index": 0,
                    "finish_reason": "tool_calls",
                    "message": {
                        "role": "assistant",
                        "tool_calls": [{
                            "id": "call_1",
                            "type": "function",
                            "function": {
                                "name": "search_files",
                                "arguments": "{\"q\":\"lint\"}"
                            }
                        }]
                    }
                }]
            })))
            .mount(&server)
            .await;

        let client = OpenAiCompatibleClient::new("test-key".to_string())
            .unwrap()
            .with_base_url(server.uri())
            .unwrap();

        let response = client.chat(request()).await.expect("success");
        let tool_calls = response.choices[0]
            .message
            .tool_calls
            .clone()
            .expect("tool calls");
        assert_eq!(tool_calls[0].function.arguments, json!({"q": "lint"}));
    }
}
