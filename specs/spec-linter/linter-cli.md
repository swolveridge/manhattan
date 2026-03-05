---
Kind: interface
Specifies:
  - spec-linter/spec-linting.md#output
---

# Linter CLI

Define command-line behavior for linting.

## Commands

- Implement `<main-exe> lint check [directory]`.
- Support `--structural-only` to skip semantic checks that require the shared `llm` module defined by `llm/invoker-api.md`.
- Support `--dry-run` to print semantic analysis prompts without invoking the shared `llm` module.
- Support `--focus path/to/spec.md` to limit scope to one spec and direct relationships.
- Support `--json` for machine-readable output.

## Exit Codes

- Exit `0` when no issues are reported.
- Exit `1` when warnings exist and no errors exist.
- Exit `2` when any errors exist.
