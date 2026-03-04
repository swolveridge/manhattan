---
Kind: feature
Specifies:
  - architecture.md#spec-linter
---

# Spec Linting

Analyze a spec corpus and report structural, semantic, and completeness issues.

## Structural Checks

- Run deterministic checks that do not require LLM calls.
- Consume the parsed spec corpus and DAG produced by the parser.
- Treat parser-reported graph-integrity issues as structural lint findings.
- Report structural findings with stable codes and source locations.

## Contradiction Detection

- Detect contradictions between related specs.
- Distinguish contradiction from valid refinement.
- Report evidence from both conflicting specs.

## Gap Detection

- Detect referenced but unspecified concepts.
- Detect missing error-handling and interface coverage where expected.

## Ambiguity Detection

- Detect vague, multi-interpretable, or undefined terms.
- Report ambiguity findings with confidence and rationale.

## Output

- Return issues with severity, category, code, message, and location when available.
- Return all discovered issues in one pass.
