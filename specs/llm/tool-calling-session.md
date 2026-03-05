---
Kind: behavioural
Specifies:
  - llm/llm-invocation.md#tool-calling-sessions
---

# Tool-Calling Session

Define deterministic orchestration for assistant tool-call workflows.

## Loop Behavior

- Treat an assistant response with `tool_calls` as a continuation step, not terminal output.
- Execute requested tools locally and append one `tool` message per tool call ID.
- Treat an assistant response without `tool_calls` as terminal output.
- Validate terminal responses using `llm/response-validation.md`.
- Continue requesting completions with the expanded message history until terminal output is produced.

## Safety and Cost Controls

- Enforce a configurable maximum tool-call count per session, with default `8` and allowed range `1..64`.
- Fail the session when the tool-call budget is exceeded.
- Fail when terminal responses are empty (`tool_calls` absent and trimmed `content` length equals `0`).

## Tool Execution Interface

- Register tools by name before starting a session.
- Use a tool execution interface that accepts `(name, arguments)` and returns either output text or a tool execution error.
- When a tool execution error occurs, append a `tool` message containing the failure details and continue the session.

## Observability

- Expose each tool call name and arguments for logging.
- Keep tool call IDs and ordering stable through the session.
