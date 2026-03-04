use std::future::Future;

use serde_json::Value;

use crate::llm::client::OpenAiCompatibleClient;
use crate::llm::error::InvocationError;
use crate::llm::types::{ChatRequest, ChatResponse, Message, Role};

pub trait ChatInvoker {
    fn chat(
        &self,
        request: ChatRequest,
    ) -> impl Future<Output = Result<ChatResponse, InvocationError>> + Send;
}

impl ChatInvoker for OpenAiCompatibleClient {
    fn chat(
        &self,
        request: ChatRequest,
    ) -> impl Future<Output = Result<ChatResponse, InvocationError>> + Send {
        OpenAiCompatibleClient::chat(self, request)
    }
}

pub async fn run_tool_call_session<I, F, E>(
    invoker: &I,
    mut request: ChatRequest,
    max_tool_calls: usize,
    mut execute_tool: F,
) -> Result<ChatResponse, InvocationError>
where
    I: ChatInvoker + Sync,
    F: FnMut(&str, &Value) -> E,
    E: Future<Output = Result<String, InvocationError>>,
{
    let mut tool_calls_used = 0usize;

    loop {
        let response = invoker.chat(request.clone()).await?;
        let first_choice = response
            .choices
            .first()
            .ok_or_else(|| InvocationError::InvalidResponse("missing choices".to_string()))?;

        let assistant_message = first_choice.message.clone();
        if let Some(tool_calls) = assistant_message.tool_calls.clone()
            && !tool_calls.is_empty()
        {
            request.messages.push(assistant_message);

            for call in tool_calls {
                tool_calls_used += 1;
                if tool_calls_used > max_tool_calls {
                    return Err(InvocationError::ToolCallLimitExceeded {
                        max: max_tool_calls,
                    });
                }

                let output = execute_tool(&call.function.name, &call.function.arguments).await?;
                request.messages.push(Message {
                    role: Role::Tool,
                    content: Some(output),
                    tool_call_id: Some(call.id),
                    tool_calls: None,
                });
            }
            continue;
        }

        if assistant_message
            .content
            .as_ref()
            .is_none_or(|content| content.trim().is_empty())
        {
            return Err(InvocationError::InvalidResponse(
                "terminal assistant message has no content".to_string(),
            ));
        }

        return Ok(response);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use serde_json::json;

    use super::*;
    use crate::llm::types::{Choice, ToolCall, ToolFunctionCall};

    struct FakeInvoker {
        responses: Mutex<VecDeque<ChatResponse>>,
    }

    impl ChatInvoker for FakeInvoker {
        fn chat(
            &self,
            _request: ChatRequest,
        ) -> impl Future<Output = Result<ChatResponse, InvocationError>> + Send {
            let response = self
                .responses
                .lock()
                .expect("lock")
                .pop_front()
                .expect("queued response");
            async move { Ok(response) }
        }
    }

    fn request() -> ChatRequest {
        ChatRequest {
            model: "openai/gpt-5".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: Some("review this".to_string()),
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
    async fn runs_until_terminal_output() {
        let first = ChatResponse {
            id: "1".to_string(),
            model: "openai/gpt-5".to_string(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: None,
                    tool_call_id: None,
                    tool_calls: Some(vec![ToolCall {
                        id: "call_1".to_string(),
                        function: ToolFunctionCall {
                            name: "read_file".to_string(),
                            arguments: json!({"path": "src/main.rs"}),
                        },
                    }]),
                },
                finish_reason: Some("tool_calls".to_string()),
            }],
            usage: None,
        };
        let second = ChatResponse {
            id: "2".to_string(),
            model: "openai/gpt-5".to_string(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: Some("final answer".to_string()),
                    tool_call_id: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        };

        let invoker = FakeInvoker {
            responses: Mutex::new(VecDeque::from([first, second])),
        };

        let response = run_tool_call_session(&invoker, request(), 4, |_name, _args| async {
            Ok("file body".to_string())
        })
        .await
        .expect("session succeeds");

        assert_eq!(response.id, "2");
    }

    #[tokio::test]
    async fn enforces_tool_call_limit() {
        let response_with_tool_call = ChatResponse {
            id: "1".to_string(),
            model: "openai/gpt-5".to_string(),
            choices: vec![Choice {
                index: 0,
                message: Message {
                    role: Role::Assistant,
                    content: None,
                    tool_call_id: None,
                    tool_calls: Some(vec![ToolCall {
                        id: "call_1".to_string(),
                        function: ToolFunctionCall {
                            name: "read_file".to_string(),
                            arguments: json!({"path": "src/main.rs"}),
                        },
                    }]),
                },
                finish_reason: Some("tool_calls".to_string()),
            }],
            usage: None,
        };

        let invoker = FakeInvoker {
            responses: Mutex::new(VecDeque::from([response_with_tool_call])),
        };

        let err = run_tool_call_session(&invoker, request(), 0, |_name, _args| async {
            Ok("file body".to_string())
        })
        .await
        .expect_err("must fail");

        assert!(matches!(
            err,
            InvocationError::ToolCallLimitExceeded { max: 0 }
        ));
    }
}
