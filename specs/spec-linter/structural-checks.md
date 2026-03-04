---
Kind: behavioural
Specifies:
  - spec-linter/spec-linting.md#structural-checks
---

# Structural Checks

Implement deterministic lint policy checks over the parsed spec corpus and DAG.

## Parser-Derived Integrity Findings

- Include parser-reported graph-integrity diagnostics in lint output.
- Do not re-implement parsing, link resolution, or cycle algorithms in the linter.

## Linter Policy Checks

- Enforce orphan policy for `feature`, `behavioural`, and `interface` specs with no valid `Specifies` targets.
- Exempt `constraint` and `context` kinds from orphan policy.
- Exempt `spec-format.md` from orphan policy by filename.
- Exempt specs marked `Root: true` from orphan policy.
- Enforce `Root: true` policy constraints:
  - at most one spec declares `Root: true`
  - only `Kind: feature` specs may declare `Root: true`
- Report specs with no markdown headings as missing required description structure.

## Scope

- Run checks over all managed specs in the target directory by default.
- Support focused checking of one spec and its direct relationships.
