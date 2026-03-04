use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvocationError {
    #[error("missing API key; provide explicit config or set OPENAI_API_KEY")]
    MissingApiKey,

    #[error("invalid base URL '{value}': {message}")]
    InvalidBaseUrl { value: String, message: String },

    #[error("network error: {0}")]
    Network(String),

    #[error("HTTP error {status}: {body}")]
    Http { status: u16, body: String },

    #[error("provider-declared error: {0}")]
    Provider(String),

    #[error("response parse error: {0}")]
    Parse(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("tool-call limit exceeded (max {max})")]
    ToolCallLimitExceeded { max: usize },
}
