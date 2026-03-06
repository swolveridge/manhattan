---
Kind: behavioural
Specifies:
  - prompt-generator/prompt-generation.md#template-filling
---

# Template Filling

Define template syntax, variable resolution, and render-time validation behavior.

## Template Syntax

- Store templates as UTF-8 text files with stable file names under component-managed prompt template directories.
- Represent placeholders with `{{variable_name}}` syntax.
- Treat variable names as case-sensitive identifiers.
- Support escaped placeholder literals so templates can include `{{` and `}}` as plain text when needed.

## Variable Contract

- Define per-template required and optional variables.
- Resolve variables from explicit render input maps.
- Permit string and structured values when renderer rules define deterministic stringification.
- Reject implicit ambient variables not declared in render input maps.

## Conditional Blocks

- Support conditional sections gated on variable presence.
- Omit gated sections entirely when the condition is false.
- Keep deterministic whitespace handling so equivalent inputs render byte-stable output.

## Missing and Unknown Variables

- Fail rendering when required variables are missing.
- Report unresolved placeholder names in error output.
- Reject unknown provided variables.

## Validation and Diagnostics

- Validate template syntax before placeholder substitution.
- Return diagnostics with template identifier and position information when available.
- Return all discoverable template validation issues in one validation pass.

## Compatibility Contract

- This spec defines no compatibility requirements for template syntax evolution.
