---
Kind: feature
Root: true
---

# System Architecture

The system keeps code aligned with declarative specs through a two-phase reconciliation model.

## Implementation Language

- Implement all first-party tools and libraries in Rust.
- Use Rust as the default language for parser, linter, LLM invoker, prompt generator, test deriver, reconciler, and traceability components.
- Introduce non-Rust implementation languages only when an explicit constraint spec permits an exception.
- Apply Rust coding and tooling conventions defined in `rust-standards.md`.

## Reconciliation Model

### Spec Reconciliation

- The system analyzes spec consistency.
- The system identifies contradictions, ambiguity, gaps, and stale statements.
- The system requests updates to maintain an internally consistent spec corpus.
    - Updates will be provided by the user in "interactive" mode
    - Updates will be provided by an LLM in "autonomous" mode

## Core Components

Each component must be a separate Rust module.

### Spec Parser and DAG Builder

- The parser reads markdown specs and builds a directed acyclic graph from `Specifies:` links.
- The parser reports structural issues such as broken links and cycles.

### Spec Linter

- The linter performs structural checks deterministically.
- The linter performs semantic checks with LLM assistance.
- The linter outputs issues with severity, location, and explanation.

### Prompt Generator

- The prompt generator builds task-specific prompts from templates and context.
- Prompt generation follows `prompt-standards.md`.
- Prompt generator behavior and interface are defined in `prompt-generator/prompt-generation.md`.

## Required Supporting Components

### LLM Invoker

- The LLM invoker provides a shared OpenAI-compatible client for all LLM-backed workflows.
- The invoker encapsulates provider transport concerns, response validation, and tool-call session handling.
- Linter, prompt generation, and reconciliation flows consume this component rather than duplicating invocation logic.

## `.specignore` Boundary

- `.specignore` marks files and directories outside reconciliation control.
- `.specignore` uses `.gitignore` pattern syntax and matching behavior.
- Ignored code is treated as an unmanaged dependency.
- Managed code interfaces with unmanaged code and non-code-accessible external systems through contracts defined in `external-integration/interface-contracts.md`.

## Source-of-Truth Principle

- Specs are the source of truth for intended behavior.
- Code is the source of truth for current running behavior.
- Reconciliation maintains alignment between intended behavior and running behavior without treating code as a disposable artifact.
