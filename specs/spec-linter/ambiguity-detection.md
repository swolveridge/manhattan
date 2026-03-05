---
Kind: behavioural
Specifies:
  - spec-linter/spec-linting.md#ambiguity-detection
  - llm/invoker-api.md#invocation-method
---

# Ambiguity Detection

Use LLM-assisted analysis to find unclear requirements.
Use the shared `llm` module contract defined by `llm/invoker-api.md` for all LLM calls.

## Analysis Scope

- Analyze the full in-scope spec corpus in one LLM invocation per lint run.
- Report ambiguity wherever it appears in the corpus.

## Required Findings

- Ambiguous statements with multiple plausible interpretations.
- Undefined domain terms.
- Vague quantifiers and unverifiable adjectives.
