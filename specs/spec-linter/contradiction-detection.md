---
Kind: behavioural
Specifies:
  - spec-linter/spec-linting.md#contradiction-detection
  - llm/invoker-api.md#invocation-method
---

# Contradiction Detection

Use LLM-assisted analysis to find incompatible requirements.
Use the shared `llm` module contract defined by `llm/invoker-api.md` for all LLM calls.

## Comparison Scope

- Analyze the full in-scope spec corpus in a single pass.
- Identify contradictions wherever they occur in the corpus, including sibling-vs-sibling and child-vs-parent conflicts.

## Reporting

- Report contradiction findings with confidence (`high|medium|low`).
- Include short evidence excerpts from both sides.
