---
Kind: feature
Specifies:
  - architecture.md#prompt-generator
---

# Prompt Generation

Define the prompt generator component used by linter, test derivation, and reconciliation workflows.

## Scope

- Build task-specific prompts from templates plus assembled spec corpus context.
- Keep prompt construction deterministic for identical inputs.
- Keep prompt rendering separate from LLM transport and invocation concerns.
- Keep implementation simple: read spec files from one directory without link-following or graph traversal.

## Template Filling

- Resolve declared template placeholders from provided input values.
- Fail when required placeholders are missing or unresolved.
- Reject unknown placeholder variables.

## Context Assembly

- Accept a single directory of specs as input context.
- Read markdown spec files in that directory recursively.
- Assemble context directly from file contents.
- Do not follow `Specifies:` links or markdown cross-references.
- Preserve explicit context block delimiters required by `prompt-standards.md`.

## Prompt Generator Interface

- Expose a reusable library API for context assembly, validation, and render.
- Accept template identifier and spec directory.
- Return one rendered prompt string.
- Ensure rendered prompt structure conforms to `prompt-standards.md`.
- Keep context block ordering deterministic.

### Error Behaviour

- Return typed errors for template resolution failures.
- Return typed errors for invalid template identifiers and unreadable spec directories.
- Return all validation failures discoverable in one pass before rendering output.
