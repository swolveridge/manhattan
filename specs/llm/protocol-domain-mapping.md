---
Kind: behavioural
Specifies:
  - llm/llm-invocation.md#protocol-and-domain-model-boundary
---

# Protocol Domain Mapping

Define mapping rules between provider protocol DTOs and component-facing domain models.

## Layering

- Keep protocol DTOs in a dedicated protocol module scoped to provider transport.
- Keep domain-model types in a separate types module used by linter, prompt generation, and reconciler code.
- Prevent direct use of protocol DTOs by non-invoker components.

## Request Mapping

- Map domain `ChatRequest` into protocol request payloads immediately before HTTP serialization.
- Map domain tool definitions into provider `function` tool schema shape.
- Preserve message order, roles, and tool-call linkage IDs through mapping.

## Response Mapping

- Parse protocol response payloads into protocol DTOs before domain conversion.
- Map first-class assistant output into domain `ChatResponse` types.
- Preserve tool-call IDs and function payloads in domain response types.

## Normalization

- Normalize tool-call argument payloads into a single domain representation regardless of whether provider payload is an object or a JSON-encoded string.
- Validate normalization failures as invocation errors with enough context for diagnostics.

## Compatibility

- Keep mapping logic explicit and test-covered.
- Treat protocol DTO changes and domain type changes as independent, linked only through mapping code.
