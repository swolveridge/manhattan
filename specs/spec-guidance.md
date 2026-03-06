---
Kind: context
---

# Spec Guidance

Practical guidance for writing specs.

## Purpose of Specs

- Treat specs as the source of truth for intended behaviour.
- Write specs as durable communication artifacts for both humans and coding agents.
- Capture decisions and externally meaningful behaviour in specs; keep mechanical implementation details in code.

## Multi-Level Refinement

- Organise specs at multiple levels of detail.
- Treat lower-level specs as refinements of higher-level specs.
- Preserve meaning across levels: detail may be added, intent must not be changed.
- Expect overlap across levels; this is refinement, not duplication, as long as levels remain consistent.

## DAG and Relationship Discipline

- Link specs in a directed acyclic graph from lower-level detail to higher-level intent using `Specifies`.
- Prefer section-level targets (`file.md#heading`) so relationships are precise and reviewable.
- Use multiple `Specifies` targets when a spec is cross-cutting or defines an interaction between features.
- Keep the graph acyclic and with valid targets; broken links and cycles are structural defects.

## How to Write Good Spec Content

- Use declarative statements of required system behaviour.
- State observable outcomes, constraints, edge cases, and failure behaviour.
- Define terms and referenced concepts; avoid introducing entities that are never specified elsewhere.
- Avoid vague language (`quickly`, `appropriately`, `efficiently`) unless made measurable.
- Include enough detail that tests can be derived from the spec.

## Cross-Cutting and Integration Expectations

- When behaviour spans multiple features, write an explicit interaction spec and link it to all relevant parents.
- Surface assumptions and boundary expectations at integration points.
- Ensure cross-feature behaviour is specified, not inferred from isolated feature specs.

## Constraint and Context Specs

- Use constraint specs for rules that limit solution space across the system (for example prompt standards, policy constraints).
- Use context specs for rationale, sequencing decisions, and adoption boundaries.
- Keep decision history and rejected alternatives in context specs to prevent re-litigating choices.

## Spec Quality Checks to Apply

- Check for contradictions between siblings and between high-level and low-level specs.
- Check for ambiguity where multiple reasonable interpretations exist.
- Check for gaps: undefined concepts, missing error handling, missing interface coverage for referenced systems.

## Authoring Workflow (Recommended)

1. Start from the relevant high-level spec section and clarify intended outcome.
2. Add or update lower-level behavioural/interface specs that refine it.
3. Add cross-cutting links and integration details where interactions exist.
4. Review for contradiction, ambiguity, gaps, and testability.
5. Confirm no structural issues in the spec graph (broken links, cycles, orphaned required specs).
