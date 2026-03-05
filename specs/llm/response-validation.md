---
Kind: behavioural
Specifies:
  - llm/llm-invocation.md#response-and-error-semantics
---

# Response Validation

Define response parsing and validation rules for robust invocation.

## Success Payload Requirements

- Parse successful responses as JSON objects before domain mapping.
- Require at least one response choice.
- For continuation steps, accept assistant responses that include one or more `tool_calls` (content may also be present).
- For terminal steps, accept assistant responses only when `tool_calls` are absent and `content` is non-empty.

## Error Payload Detection

- Detect and surface top-level `error` payloads even on HTTP 200 responses.
- Detect and surface choice-level error payloads when present.
- Treat `finish_reason: error` responses as invocation failures.
- Classify and report validation and provider-declared failures using `llm/invocation-errors.md`.

## Parse and Shape Validation

- Treat truncated or non-JSON response bodies as invocation failures.
- Treat schema-mismatched payloads as invocation failures with raw body context.
