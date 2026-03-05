---
Kind: feature
Specifies:
  - architecture.md#prompt-generator
---

# Prompt Generation

Define the prompt generator component used by linter and reconciliation workflows.

## Inputs

- Accept a prompt template identifier.
- Accept structured context payloads (spec corpus excerpts, file paths, constraints, and task parameters).

## Output

- Return one rendered prompt string per invocation.
- Rendered prompts must conform to `prompt-standards.md`.

## Behavior

- Resolve template placeholders from provided context values.
- Fail with a typed error when required placeholders are missing.
