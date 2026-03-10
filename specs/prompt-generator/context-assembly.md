---
Kind: behavioural
Specifies:
  - prompt-generator/prompt-generation.md#context-assembly
---

# Context Assembly

Define how prompt context is selected, ordered, and labelled.

## Input Scope

- Accept a directory of specs to include.
- Accept no additional context sources.
- Treat the provided directory as the complete input corpus for prompt context.

## Corpus Loading

- Read all `.md` files under the provided directory recursively.
- Ignore non-markdown files.
- Sort loaded spec files by relative path before assembly for deterministic ordering.

## Link Handling

- Do not follow `Specifies:` references.
- Do not resolve markdown cross-references.
- Do not infer parent, sibling, or child relationships between specs.

## Delimiters and Labels

- Separate instruction text and each context block with explicit delimiters.
- Label each block with file path provenance.
- Keep delimiter conventions aligned with `prompt-standards.md#context-delimiters`.

## Error Behaviour

- Fail when the provided directory does not exist or cannot be read.
- Return typed errors with file-system provenance when available.
