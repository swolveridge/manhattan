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

## Two-Phase Reconciliation Model

### Phase 1: Spec Reconciliation

- The system analyzes spec consistency before code changes.
- The system identifies contradictions, ambiguity, gaps, and stale statements.
- The system updates or requests updates to maintain an internally consistent spec corpus.

### Phase 2: Code Reconciliation

- The system treats the consistent spec corpus as desired state.
- The system updates code and tests to satisfy the specs.
- The system verifies results through independently derived tests and analysis passes.

## Core Components

Each component should be a separate rust module.

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

### Test Deriver

- The test deriver independently generates executable tests from specs.
- Test derivation is isolated from implementation details except declared interfaces.

### Reconciler

- The reconciler proposes and applies code changes to satisfy target specs.
- The reconciler runs verification loops with test derivation and execution.
- The reconciler reports proportionality and residue findings.

### Traceability

- Traceability maps specs to code and code to specs as a many-to-many relationship.
- Traceability is derivable on demand and may be cached as a non-authoritative optimization.

## Required Supporting Components

### LLM Invoker

- The LLM invoker provides a shared OpenAI-compatible client for all LLM-backed workflows.
- The invoker encapsulates provider transport concerns, response validation, and tool-call session handling.
- Linter, prompt generation, and reconciliation flows consume this component rather than duplicating invocation logic.

## `.specignore` Boundary

- `.specignore` marks files and directories outside reconciliation control.
- Ignored code is treated as an unmanaged dependency.
- Managed code interfaces with unmanaged code through interface specs.

## Source-of-Truth Principle

- Specs are the source of truth for intended behavior.
- Code is the source of truth for current running behavior.
- Reconciliation maintains alignment between intended behavior and running behavior without treating code as a disposable artifact.
