---
Kind: behavioural
Specifies:
  - architecture.md#reconciler
---

# Reconciliation Reporting

Define required reconciler findings for verification and residual risk tracking.

## Proportionality Findings

- Report the relationship between changed surface area and targeted spec requirements.
- Each proportionality finding object must include:
  - `id` (string);
  - `target_requirements` (array of spec addresses);
  - `changed_files` (array of repository-relative paths);
  - `change_scope_summary` (string).

## Residue Findings

- Report unresolved items remaining after a reconciliation attempt.
- Classify each residue item as one of:
  - blocked by missing spec detail;
  - blocked by external dependency;
  - deferred by explicit scope decision;
  - verification failure requiring additional changes.
- Each residue finding object must include:
  - `id` (string);
  - `classification` (one of the residue classes above);
  - `description` (string);
  - `next_action` (string).

## Output Requirements

- Emit findings in JSON as the machine-readable format with:
  - `proportionality_findings` (array of proportionality finding objects);
  - `residue_findings` (array of residue finding objects).
- Emit a human-readable summary as plain text with sections `Proportionality` and `Residue`.
- Preserve stable identifiers for requirement and residue references within one reconciliation run.
