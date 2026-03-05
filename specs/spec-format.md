---
Kind: context
---

# Spec Format

This document defines the canonical spec format for this repository. It is the only meta-level spec and is excluded from reconciliation by name via `.specignore`.

## Scope

- All managed specs live in the `specs/` directory.
- Spec files must use the `.md` extension.
- Subdirectories under `specs/` are allowed for human organisation.
- Directory structure under `specs/` has no semantic meaning.

## Front Matter

Each spec file must start with YAML front matter containing:

- `Kind: feature | behavioural | interface | constraint | context`
- `Specifies: [...]` (optional for `constraint` and `context`; conditionally required for other kinds by lint rules)
- `Root: true` (optional; marks the single DAG root spec)

`Kind` meanings:

- `feature`: user- or system-visible capabilities
- `behavioural`: detailed behaviour and edge-case rules
- `interface`: APIs, contracts, schemas, and CLI surfaces
- `constraint`: cross-cutting constraints that limit solution space
- `context`: rationale, background, and adoption decisions

`Specifies` format:

- `Specifies` is a YAML list of one or more section targets.
- Each target must be `relative/path/to/file.md#heading-slug`, relative to `specs/`.
- Multiple targets are allowed for cross-cutting specs.
- If a spec has no parent, `Specifies` may be omitted only when `Root: true` is set.

`Root` format:

- `Root` is optional and must be a boolean when present.
- `Root: true` marks a spec as the DAG root.
- At most one spec in the corpus may declare `Root: true`.
- `Root: true` may only appear on `Kind: feature` specs.

## Section Addressing

- A spec section is addressed as `file.md#heading-slug`.
- Heading slugs follow GitHub-flavoured markdown conventions.
- Cross-references use standard markdown links.

## `Specifies` Convention

- Parent links are declared in front matter, not in section bodies.
- `Specifies` links define DAG edges from this spec to higher-level spec sections.
- Cross-cutting concerns are represented by multiple `Specifies` targets.

Example:

```markdown
---
Kind: behavioural
Specifies:
  - authentication.md#password-reset
---

# Password Reset Token Lifetime

The system invalidates a password reset token 30 minutes after issuance.
```

## Writing Style

- Specs are imperative.
- Required style: "Implement X", "Reject Y", "Return Z when condition C holds."
- Avoid declarative preference statements such as "The system does X."
- Imperative statements must still define verifiable outcomes and constraints, not low-level mechanics.

## Spec vs Code Boundary

- Put decisions in specs when they affect externally observable behavior, declared interfaces, safety constraints, or policy constraints.
- Put mechanics in code when behaviour is a direct consequence of accepted decisions.
- Include interfaces and externally observable behaviour in specs.
- Exclude incidental implementation details unless they are constraints.

## Validation Expectations

- Feature, behavioural, and interface specs participate in the reconciliation DAG.
- Constraint and context specs may float without `Specifies:` links.
- A `feature` spec with `Root: true` is exempt from orphan detection.
