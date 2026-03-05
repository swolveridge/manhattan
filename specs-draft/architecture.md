# System Architecture

...

## Two-Phase Reconciliation Model

### Phase 1: Spec Reconciliation

- The system analyzes spec consistency before code changes.
- The system identifies contradictions, ambiguity, gaps, and stale statements.
- The system requests updates to maintain an internally consistent spec corpus.

### Phase 2: Code Reconciliation

- The system treats the consistent spec corpus as desired state.
- The system updates code and tests to satisfy the specs.
- The system verifies results through independently derived tests and analysis passes.

## Core Components

...

### Test Deriver

- The test deriver independently generates executable tests from specs.
- Test derivation is isolated from implementation details except declared interfaces.
- Test deriver behavior and interfaces are defined in `test-deriver/test-derivation.md`.

### Reconciler

- The reconciler proposes and applies code changes to satisfy target specs.
- The reconciler runs verification loops with test derivation and execution.
- Reconciler execution behavior is defined in `reconciler/reconciliation-execution.md`.
- The reconciler reports findings defined in `reconciler/reconciliation-reporting.md`.

### Traceability

- Traceability maps specs to code and code to specs as a many-to-many relationship.
- Traceability is derivable on demand and may be cached as a non-authoritative optimization.
- Traceability behavior and interfaces are defined in `traceability/traceability-mapping.md`.

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
