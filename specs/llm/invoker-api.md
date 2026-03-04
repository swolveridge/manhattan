---
Kind: interface
Specifies:
  - llm/llm-invocation.md#reuse-contract
---

# LLM Invoker API

Define the shared Rust API for OpenAI-compatible LLM invocation.

## Construction

- Construct an invoker with API key and optional base URL override.
- Expose base URL override so callers can target compatible providers.

## Invocation Method

- Expose an async chat-completion method taking domain-model request types.
- Domain request types include `model`, ordered `messages`, and optional fields:
  - `response_format`
  - `tools`
  - `tool_choice`
  - `temperature`
  - `max_tokens`
  - `reasoning_effort`

## Response Types

- Return domain response types containing `id`, `model`, `choices`, and `usage`.
- Preserve assistant tool-call payloads and IDs when present.

## Protocol Types

- Keep protocol-layer request/response DTOs internal to the invoker transport boundary.
- Convert domain request types to protocol DTOs before serialization.
- Convert protocol response DTOs to domain response types before returning to callers.

## Error Contract

- Return typed invocation errors for transport, HTTP, parse, and provider-declared failures.
- Include enough context for callers to log and classify failures.

## Compatibility

- Keep DTO fields stable for downstream callers.
- Permit additive, backward-compatible extension for new compatible API fields.
