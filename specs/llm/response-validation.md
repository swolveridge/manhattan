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
- Accept assistant responses with either content or tool calls.

## Error Payload Detection

- Detect and surface top-level `error` payloads even on HTTP 200 responses.
- Detect and surface choice-level error payloads when present.
- Treat `finish_reason: error` responses as invocation failures.

## Parse and Shape Validation

- Treat truncated or non-JSON response bodies as invocation failures.
- Treat schema-mismatched payloads as invocation failures with raw body context.
