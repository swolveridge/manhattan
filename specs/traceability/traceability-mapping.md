---
Kind: feature
Specifies:
  - architecture.md#traceability
---

# Traceability Mapping

Define how the system maps specs to code and code back to specs.

## Mapping Model

- Maintain many-to-many links between spec statements and code artifacts.
- Support both directions: spec-to-code and code-to-spec queries.

## Derivation

- Derive mappings from repository contents and declared spec references.
- Treat derived mappings as authoritative for the current run.

## Caching

- Cache mappings as an optional optimization.
- Treat cached mappings as non-authoritative and recompute when cache is stale or unavailable.
