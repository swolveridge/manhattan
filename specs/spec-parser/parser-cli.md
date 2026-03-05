---
Kind: interface
Specifies:
  - spec-parser/spec-parsing.md#parser-interface
---

# Parser CLI

Define command-line behavior for parser-only validation.

## Commands

- Implement `<main-exe> parse check [directory]`.
- Default `[directory]` to `specs/` when omitted.
- Support `--json` for machine-readable output.

## Output

- Print a summary with diagnostic counts in human-readable mode.
- Print diagnostics with severity, code, message, and location when available.

## Exit Codes

- Exit `0` when no diagnostics with severity `error` are reported.
- Exit `2` when any diagnostics with severity `error` are reported.
