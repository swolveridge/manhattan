---
Kind: interface
Specifies:
  - test-deriver/test-derivation.md#outputs
---

# Test Deriver API

Define the interface for deriving tests from specs.

## Invocation

- Expose a function that accepts in-scope spec inputs and derivation options.
- Return derived test artifacts and traceability metadata in one result object.

## Derived Test Artifact Contract

- Each derived test artifact includes:
  - target language identifier;
  - file path recommendation;
  - executable test content;
  - referenced source spec addresses.

## Failure Contract

- Return typed errors for invalid inputs, unsupported target language, and derivation failures.
