---
Kind: constraint
---

# Spec Standards

Define required quality standards for managed specs in this repository.

Specs are written in British English.

## Core Quality Requirements

- Specs must not contain contradictions, as defined by [spec-linter/contradiction-detection.md](spec-linter/contradiction-detection.md).
- Specs must not contain ambiguity, as defined by [spec-linter/ambiguity-detection.md](spec-linter/ambiguity-detection.md).
- Specs must not contain gaps, as defined by [spec-linter/gap-detection.md](spec-linter/gap-detection.md).

## Enforcement

- `lint check` must evaluate managed specs against contradiction, ambiguity, and gap detection rules.
- Any finding in these categories must be treated as a standards violation until resolved or explicitly exempted by policy.
