---
Kind: feature
Specifies:
  - architecture.md#spec-linter
  - llm/invoker-api.md#invocation-method
  - prompt-generator/prompt-generation.md#prompt-generator-interface
---

# Spec Linting

Analyze a spec corpus and report structural, semantic, and completeness issues.

## Structural Checks

- Run deterministic checks that do not require LLM calls.
- Consume the parsed spec corpus and DAG produced by the parser.
- Treat graph diagnostics defined by `spec-parser/dag-construction.md` as structural lint findings.
- Report structural findings with stable codes and source locations.

## Contradiction Detection

- Detect contradictions across the full in-scope spec corpus.
- Assemble contradiction-analysis prompts through the shared prompt generator component defined by `prompt-generator/prompt-generation.md`.
- Run contradiction detection through the shared `llm` module defined by `llm/invoker-api.md`.
- Treat a mandatory requirement as any statement using imperative or normative requirement language (`must`, `must not`, `required`, `shall`) in the in-scope corpus.
- Classify as valid refinement when one statement narrows scope, adds constraints, or specializes behavior without negating any mandatory requirement from another in-scope statement.
- Classify as contradiction when two in-scope statements impose incompatible mandatory requirements for overlapping scope.
- Report evidence from both conflicting specs.

## Gap Detection

- Detect referenced but unspecified concepts.
- Assemble gap-analysis prompts through the shared prompt generator component defined by `prompt-generator/prompt-generation.md`.
- Run gap detection through the shared `llm` module defined by `llm/invoker-api.md`.
- Detect missing error-handling and interface coverage where expected.

## Ambiguity Detection

- Detect vague, multi-interpretable, or undefined terms.
- Assemble ambiguity-analysis prompts through the shared prompt generator component defined by `prompt-generator/prompt-generation.md`.
- Run ambiguity detection through the shared `llm` module defined by `llm/invoker-api.md`.
- Analyze the full in-scope corpus in one pass.
- Report ambiguity findings with confidence and rationale.

## Output

- Return issues with severity, category, code, message, and location when available.
- Return all discovered issues in one pass.
