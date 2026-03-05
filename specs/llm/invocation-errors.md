---
Kind: interface
Specifies:
  - llm/invoker-api.md#error-contract
---

# Invocation Errors

Define typed error variants and required context for LLM invocation failures.

## Error Variants

- `Transport`: network or client transport failures before a valid HTTP response.
- `Http`: non-success HTTP status responses.
- `Parse`: response body parse failures, including malformed or truncated JSON.
- `Provider`: provider-declared error payloads, including HTTP-200 encoded errors.
- `Validation`: structurally valid payloads that violate response contract rules.

## Required Fields

- Every invocation error includes:
  - `variant`;
  - `provider`;
  - `model`;
  - `error_code`.
- Include `request_id` when provider metadata exposes it.
- Include `http_status` for `Http` and provider-declared HTTP-level failures.
- Include `raw_excerpt` when raw payload context is available and safe to log.

## Classification Contract

- Error variants are mutually exclusive per failure event.
- Error handling paths may derive retry policy from `variant` and `error_code`.
