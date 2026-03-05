---
Kind: interface
Specifies:
  - spec-linter/spec-linting.md#output
---

# Linter CLI

Define command-line behavior for linting.

## Commands

- Implement `<main-exe> lint check [directory]`.
- Default `[directory]` to `specs/` when omitted.
- Support `--structural-only` to skip semantic checks that require the shared `llm` module defined by `llm/invoker-api.md`.
- Support `--dry-run` to print semantic analysis prompts without invoking the shared `llm` module.
- Support `--focus path/to/spec.md` to limit scope to one spec and direct relationships.
- Support `--json` for machine-readable output.

## Output

- Print human-readable summary output when `--json` is not set.
- Include issue counts and a readable list of discovered issues with file location when available.

## Diagnostics Contract

- Print each issue with severity, code, message, and file location when available.
- Keep issue codes stable across patch versions.
- Sort issues by file path, then line number when present.

## Structural Pass Behavior

- During `--structural-only`, continue parsing after recoverable file-level failures and report all recoverable issues.
- Reuse the parser library API for parsing and graph validation behavior.

## Exit Codes

- Exit `0` when no issues are reported.
- Exit `1` when warnings exist and no errors exist.
- Exit `2` when any errors exist.
