---
Kind: context
---

# Bootstrapping Strategy

This spec records why the system is built in stages and why each stage is ordered as it is.

## Build Order

- Stage 0 defines shared spec conventions before code exists.
- Stage 1 builds parsing and DAG foundations required by all later components.
- Stage 2 builds linting to validate spec quality before reconciliation touches code.
- Stage 3 extracts prompt generation as shared infrastructure.
- Stage 4 builds independent test derivation for verification.
- Stage 5 builds reconciliation after validation and prompting foundations are stable.
- Stage 6 builds traceability once scale pressure and residue analysis demand it.
- Stage 7 applies reconciliation to the system itself incrementally.

## Rationale for Incremental Adoption

- The system delivers useful capabilities early (parser, linter) before full autonomy.
- Each stage increases verification depth, reducing risk for the next stage.
- Order prioritizes learning loops on low-risk components before high-impact code modification.

## Self-Bootstrapping Position

- The system is expected to eventually manage its own codebase.
- Full self-management is deferred until parser, linter, test derivation, and reconciliation are proven through repeated cycles.
- Manual review remains required during early self-reconciliation.

## Alternatives Considered

### Immediate Self-Reconciliation

- Rejected because the required verifier stack does not yet exist at project start.
- Building reconciler-first increases risk of unverified, low-trust changes.

### Big-Bang Managed Boundary

- Rejected because brownfield codebases adopt more safely through subsystem islands.
- Managed and unmanaged boundaries can coexist while interface specs define contracts.

## Adoption Model

- Managed islands are introduced around bounded subsystems.
- Each island gains specs, parser visibility, lint coverage, and reconciliation in sequence.
- Islands can merge when shared boundaries become fully spec'd.
