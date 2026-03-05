---
Kind: feature
Specifies:
  - architecture.md#test-deriver
---

# Test Derivation

Define how tests are derived from specs without depending on implementation internals.

## Inputs

- Consume in-scope specs and their resolved cross-spec references.
- Consume declared interface contracts used by the target feature.
- Do not consume implementation-private modules, symbols, or file layout details.

## Outputs

- Produce executable tests that assert required behavior from the input specs.
- Emit traceability metadata linking each generated test to one or more source spec statements.
- Expose output interfaces defined in `test-deriver/test-deriver-api.md`.

## Isolation Rules

- Derive tests from behavioral requirements and interface contracts only.
- Reject derivation plans that require private implementation knowledge.

## Quality Rules

- Use deterministic assertions by default.
- Use heuristic checks only when no deterministic oracle exists for the requirement.
- Include negative-path tests when error behavior is specified.
