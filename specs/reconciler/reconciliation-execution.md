---
Kind: feature
Specifies:
  - architecture.md#reconciler
---

# Reconciliation Execution

Define core reconciler behavior for proposing, applying, and verifying code changes.

## Inputs

- Accept target spec scope and current repository state.
- Accept execution policy inputs (for example dry-run mode and maximum iteration count).

## Reconciliation Loop

- Derive candidate changes that satisfy target specs.
- Apply candidate changes in a working tree.
- Run verification with derived tests and required analysis checks.
- Repeat until verification succeeds or execution policy limits are reached.

## Outputs

- Emit either a verified change set or a failure result with residue items.
- Emit reporting artifacts defined by `reconciler/reconciliation-reporting.md`.
