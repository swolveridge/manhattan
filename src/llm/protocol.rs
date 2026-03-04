use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::llm::error::InvocationError;
use crate::llm::types::{
    ChatRequest, ChatResponse, Choice, Message, Role, ToolCall, ToolChoice, ToolDefinition,
    ToolFunctionCall, Usage,
};

#[derive(Debug, Serialize)]
pub(crate) struct ChatRequestBody {
    pub model: String,
    pub messages: Vec<ProtocolMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<crate::llm::types::ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ProtocolToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

impl From<ChatRequest> for ChatRequestBody {
    fn from(request: ChatRequest) -> Self {
        Self {
            model: request.model,
            messages: request
                .messages
                .into_iter()
                .map(ProtocolMessage::from)
                .collect(),
            response_format: request.response_format,
            tools: request.tools.map(|tools| {
                tools
                    .into_iter()
                    .map(ProtocolToolDefinition::from)
                    .collect()
            }),
            tool_choice: request.tool_choice,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            reasoning_effort: request.reasoning_effort,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatResponseBody {
    pub id: String,
    pub model: String,
    pub choices: Vec<ProtocolChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

impl TryFrom<ChatResponseBody> for ChatResponse {
    type Error = InvocationError;

    fn try_from(value: ChatResponseBody) -> Result<Self, Self::Error> {
        let choices = value
            .choices
            .into_iter()
            .map(Choice::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ChatResponse {
            id: value.id,
            model: value.model,
            choices,
            usage: value.usage,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProtocolChoice {
    pub index: u32,
    pub message: ProtocolMessage,
    pub finish_reason: Option<String>,
}

impl TryFrom<ProtocolChoice> for Choice {
    type Error = InvocationError;

    fn try_from(value: ProtocolChoice) -> Result<Self, Self::Error> {
        Ok(Choice {
            index: value.index,
            message: Message::try_from(value.message)?,
            finish_reason: value.finish_reason,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProtocolMessage {
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ProtocolToolCall>>,
}

impl From<Message> for ProtocolMessage {
    fn from(message: Message) -> Self {
        Self {
            role: message.role,
            content: message.content,
            tool_call_id: message.tool_call_id,
            tool_calls: message
                .tool_calls
                .map(|calls| calls.into_iter().map(ProtocolToolCall::from).collect()),
        }
    }
}

impl TryFrom<ProtocolMessage> for Message {
    type Error = InvocationError;

    fn try_from(message: ProtocolMessage) -> Result<Self, Self::Error> {
        let tool_calls = message
            .tool_calls
            .map(|calls| {
                calls
                    .into_iter()
                    .map(ToolCall::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;

        Ok(Self {
            role: message.role,
            content: message.content,
            tool_call_id: message.tool_call_id,
            tool_calls,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProtocolToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ProtocolToolFunctionCall,
}

impl From<ToolCall> for ProtocolToolCall {
    fn from(call: ToolCall) -> Self {
        Self {
            id: call.id,
            tool_type: "function".to_string(),
            function: ProtocolToolFunctionCall {
                name: call.function.name,
                arguments: ValueOrStringJson::Value(call.function.arguments),
            },
        }
    }
}

impl TryFrom<ProtocolToolCall> for ToolCall {
    type Error = InvocationError;

    fn try_from(call: ProtocolToolCall) -> Result<Self, Self::Error> {
        if call.tool_type != "function" {
            return Err(InvocationError::InvalidResponse(format!(
                "unsupported tool type '{}'",
                call.tool_type
            )));
        }

        Ok(Self {
            id: call.id,
            function: ToolFunctionCall {
                name: call.function.name,
                arguments: call.function.arguments.normalize().map_err(|err| {
                    InvocationError::InvalidResponse(format!(
                        "failed to normalize tool arguments: {err}"
                    ))
                })?,
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProtocolToolFunctionCall {
    pub name: String,
    pub arguments: ValueOrStringJson,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProtocolToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ProtocolToolFunctionDefinition,
}

impl From<ToolDefinition> for ProtocolToolDefinition {
    fn from(value: ToolDefinition) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: ProtocolToolFunctionDefinition {
                name: value.name,
                description: value.description,
                parameters: value.parameters,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ProtocolToolFunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ValueOrStringJson {
    String(String),
    Value(Value),
}

impl<'de> Deserialize<'de> for ValueOrStringJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(raw) => Ok(ValueOrStringJson::String(raw)),
            other => Ok(ValueOrStringJson::Value(other)),
        }
    }
}

impl Serialize for ValueOrStringJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ValueOrStringJson::String(value) => serializer.serialize_str(value),
            ValueOrStringJson::Value(value) => value.serialize(serializer),
        }
    }
}

impl ValueOrStringJson {
    pub(crate) fn normalize(&self) -> Result<Value, serde_json::Error> {
        match self {
            Self::Value(value) => Ok(value.clone()),
            Self::String(raw) => serde_json::from_str(raw),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn normalize_tool_args_from_object() {
        let call = ProtocolToolCall {
            id: "call_1".to_string(),
            tool_type: "function".to_string(),
            function: ProtocolToolFunctionCall {
                name: "search".to_string(),
                arguments: ValueOrStringJson::Value(json!({"q": "rust"})),
            },
        };

        let mapped = ToolCall::try_from(call).expect("maps");
        assert_eq!(mapped.function.arguments, json!({"q": "rust"}));
    }

    #[test]
    fn normalize_tool_args_from_json_string() {
        let call = ProtocolToolCall {
            id: "call_1".to_string(),
            tool_type: "function".to_string(),
            function: ProtocolToolFunctionCall {
                name: "search".to_string(),
                arguments: ValueOrStringJson::String("{\"q\":\"rust\"}".to_string()),
            },
        };

        let mapped = ToolCall::try_from(call).expect("maps");
        assert_eq!(mapped.function.arguments, json!({"q": "rust"}));
    }
}
