---
Kind: feature
Specifies:
  - architecture.md#llm-invoker
---

# LLM Invocation

Provide a reusable client layer for LLM-backed components that call OpenAI-compatible endpoints.

## Provider Compatibility

- Target OpenAI-compatible `POST /chat/completions` APIs.
- Default base URL to `https://api.openai.com/v1`.
- Allow overriding base URL for compatible providers.
- Treat model identifiers as opaque strings and pass them through unchanged.

## Request Model

- Accept chat `messages` with role-aware payloads (`system`, `user`, `assistant`, `tool`).
- Support optional generation controls (`temperature`, `max_tokens`, `reasoning_effort`).
- Support optional structured-output controls via `response_format`.
- Support optional tool-call controls via `tools` and `tool_choice`.

## Protocol and Domain Model Boundary

- Keep provider wire-format DTOs isolated in protocol-layer types.
- Expose cleaner domain-model request/response types to other components.
- Map domain-model types to protocol DTOs at request boundary and map protocol DTOs back at response boundary.
- Keep provider-specific fields and serialization quirks out of component-facing domain models.
- Normalize known protocol shape variance (for example tool-call argument payloads serialized as JSON strings vs JSON objects) during mapping.

## Tool-Calling Sessions

- Support iterative request/response workflows where assistant messages can request tool calls.
- Return each assistant tool-call request with stable IDs for local tool execution.
- Allow callers to append tool results and continue the same conversation state.

## Response and Error Semantics

- Return model response metadata (`id`, `model`, `choices`, `usage`) when available.
- Reject non-success HTTP responses as invocation errors.
- Reject malformed or truncated JSON responses as invocation errors.
- Reject error payloads encoded in otherwise-success responses.
- Reject responses with no actionable output (no content and no tool calls).

## Reuse Contract

- Expose this capability as a shared library for linter, prompt generation, and reconciler workflows.
- Keep provider-specific transport details encapsulated in this component.
