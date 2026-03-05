---
Kind: behavioural
Specifies:
  - architecture.md#reconciler
---

# Reconciliation Reporting

Define required reconciler findings for verification and residual risk tracking.

## Proportionality Findings

- Report the relationship between changed surface area and targeted spec requirements.
- Include:
  - targeted requirement references;
  - changed files list;
  - change scope summary describing why each change is necessary.

## Residue Findings

- Report unresolved items remaining after a reconciliation attempt.
- Classify each residue item as one of:
  - blocked by missing spec detail;
  - blocked by external dependency;
  - deferred by explicit scope decision;
  - verification failure requiring additional changes.
- Include next required action for each residue item.

## Output Requirements

- Emit findings in machine-readable form and human-readable summary form.
- Preserve stable identifiers for requirement and residue references within one reconciliation run.
