---
Kind: feature
Specifies:
  - architecture.md#specignore-boundary
---

# External Integration Interface Contracts

Define contract requirements for integration points outside managed in-scope code.

## External Integration Definition

- Treat as external integration any dependency that is outside the managed reconciliation boundary.
- External integrations include:
  - code paths excluded by `.specignore`;
  - remote or third-party services accessed over network boundaries;
  - platform facilities that are not code-accessible in the managed repository.

## Contract Requirement

- Every external integration used by managed code must have an interface contract spec.
- Contracts must define:
  - operations or endpoints used;
  - required inputs and expected outputs;
  - error behavior and retry or failure expectations;
  - versioning or compatibility assumptions when relevant.

## Gap Detection Expectation

- Lint gap detection reports an issue when an external integration reference has no corresponding interface contract spec.
