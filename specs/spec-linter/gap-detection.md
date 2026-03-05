---
Kind: behavioural
Specifies:
  - spec-linter/spec-linting.md#gap-detection
  - llm/invoker-api.md#invocation-method
---

# Gap Detection

Use LLM-assisted analysis to find missing specification coverage.
Use the shared `llm` module contract defined by `llm/invoker-api.md` for all LLM calls.

## Required Findings

- Referenced concepts without defining specs.
- Missing error-case behavior for interfaces and features.
- External integration references lacking contracts defined by `external-integration/interface-contracts.md`.
