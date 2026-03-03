---
Kind: interface
Specifies:
  - spec-parsing.md#parser-interface
---

# Parser CLI

Define a minimal parser command-line interface as a subcommand of the main executable.

## Command Surface

- Implement parser commands under the main executable.
- Implement `<main-exe> parse check [directory]` to parse the target directory.
- Default `[directory]` to `specs/` when omitted.

## Output

- Print human-readable summary output.
- Include counts for files parsed, nodes, edges, warnings, and errors.
- Include a readable list of discovered issues with file location when available.

## Diagnostics Contract

- Print each diagnostic with severity, code, message, and file location when available.
- Keep diagnostic codes stable across patch versions.
- Sort diagnostics by file path then by line number when present.

## Exit Behaviour

- Return exit code `0` in all parser-result cases, including when warnings or errors are present.
- Use output content, not process exit status, to communicate discovered issues.

## Operational Behaviour

- Continue parsing after recoverable file-level failures and report all recoverable issues.
- Reuse the parser library API for all parsing and validation behaviour.
