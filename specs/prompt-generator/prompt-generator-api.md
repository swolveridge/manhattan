---
Kind: interface
Specifies:
  - prompt-generator/prompt-generation.md#prompt-generator-interface
---

# Prompt Generator API

Define a reusable programmatic interface for prompt template rendering.

## API Shape

- Expose an in-process library API.
- Expose operations for template discovery, context assembly, render, and validation.
- Avoid CLI-only coupling so workflows can invoke prompt generation directly.

## Input Contract

- Accept template identifier.
- Accept specs directory path.
- Accept no auxiliary context payloads.

## Output Contract

- Return rendered prompt text.
- Return deterministic output for identical input payloads.

## Error Contract

- Return typed errors for unknown templates, unreadable specs directory, and template validation failures.
- Include source provenance in errors when available.
- Return all discoverable validation errors in one pass before render failure.

## Compatibility Contract

- This spec defines no compatibility requirements for prompt generator API evolution.
