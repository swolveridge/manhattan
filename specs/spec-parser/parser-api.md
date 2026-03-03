---
Kind: interface
Specifies:
  - spec-parser/spec-parsing.md#parser-interface
---

# Parser API

Define a reusable library interface for spec parsing and graph construction.

## API Shape

- Expose a function that accepts a directory path.
- Return parsed specs, resolved graph nodes, resolved graph edges, and diagnostics in a single result object.
- Avoid CLI-only coupling; allow direct in-process invocation by other components.

## Input Contract

- Accept the path to the repository `specs/` directory.
- Allow caller-provided options for strict mode and warning handling.

## Output Contract

- Return node identifiers as canonical addresses (`file-name.md` or `file-name.md#heading-slug`).
- Return edges as source-target address pairs.
- Return diagnostics with severity, message, and source location.

## Diagnostics Contract

- Include file path in every diagnostic when available.
- Include line and column when available.
- Distinguish errors from warnings.
- Return all diagnostics from one parse pass; do not stop at first failure.

## Compatibility Contract

- Keep output stable enough for linter and reconciler integration.
- Version breaking API changes explicitly.
