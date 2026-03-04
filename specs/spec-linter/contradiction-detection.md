---
Kind: behavioural
Specifies:
  - spec-linter/spec-linting.md#contradiction-detection
---

# Contradiction Detection

Use LLM-assisted analysis to find incompatible requirements.

## Comparison Scope

- Compare sibling specs that specify the same parent section.
- Compare child specs against their parent requirements.

## Reporting

- Report contradiction findings with confidence (`high|medium|low`).
- Include short evidence excerpts from both sides.
