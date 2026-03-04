---
Kind: behavioural
Specifies:
  - llm/llm-invocation.md#provider-compatibility
---

# OpenAI-Compatible Transport

Define request/transport behavior for OpenAI-compatible chat completion calls.

## Endpoint and Authentication

- Send requests to `{base_url}/chat/completions`.
- Send bearer authentication in `Authorization` headers.
- Send JSON payloads with `Content-Type: application/json`.

## Base URL and Credential Policy

- Resolve API keys from explicit configuration first, then environment configuration.
- Resolve base URL from explicit configuration first, then environment configuration.
- Fail fast when no API key is available.

## Request Serialization

- Serialize only set optional fields; omit unset fields.
- Preserve message ordering exactly as provided by caller.
- Preserve caller-provided model string exactly.

## Transport Failure Reporting

- Surface network and timeout failures with actionable error context.
- Include HTTP status and response body for non-success responses.
