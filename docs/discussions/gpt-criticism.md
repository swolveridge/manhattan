Key things to interrogate in this design are (a) what you can make *deterministic* vs “LLM judgement”, (b) what you treat as *authoritative* when artefacts disagree, and (c) how you stop the loop from “converging” to nonsense that merely looks consistent.

## 1) The biggest conceptual risk: “spec consistency” is underspecified

Phase 1 ends when the spec DAG is “internally consistent and the human is satisfied”. That hides a hard problem: *consistent with what semantics*?

Two specs can be mutually consistent in plain English while still leaving huge degrees of freedom (performance, security, timing, ordering, failure modes, concurrency). An LLM linter will tend to say “no contradictions” unless you formalise what counts.

Practical suggestion: define **consistency levels** that are progressively stronger, with explicit acceptance criteria:

* **Structural consistency** (links, DAG, orphans) — deterministic.
* **Terminology consistency** (entities, nouns, shared glossary) — semi-deterministic (LLM assisted but checkable).
* **Behavioural completeness** for a given feature (enumerated scenarios) — measurable.
* **Non-functional constraints coverage** (latency, rate limits, privacy, audit) — measurable.
* **Interaction coverage** (explicit interactions table / matrix) — measurable.

Without something like this, Phase 1 can “complete” while still being dangerously underspecified, and Phase 2 will confidently fill in the gaps.

## 2) Spec DAG: headings-as-nodes is workable, but you’ll want stable IDs

Using `file.md#heading` as an address will churn under renames, heading tweaks, and reorganisation, which will create noisy diffs and fragile traceability.

Consider introducing a **stable section identifier** that survives edits, e.g.

* A front-matter `ID: auth.password_reset` per section, or
* HTML comments like `<!-- spec:id=auth.password_reset -->` above a heading.

You can still keep human-friendly headings; the ID is the durable join key across:

* DAG edges (`Specifies: …`)
* Test tags (“verifies spec:id=…”)
* Traceability
* Historical continuity in git.

This also makes “spec evolution” less painful because you can refactor documents without breaking the graph.

## 3) “Spec the decisions, not the mechanics” needs guardrails

It’s a good cut, but you’ll need an operational rule for where teams will disagree.

Two failure modes:

* **Over-specification**: you end up writing pseudo-code in prose (hard to lint, brittle, bikesheddy).
* **Under-specification**: reconciliation invents policy decisions and you only notice later.

A guardrail that helps: require each behavioural spec section to include:

* **Observable outcomes** (what an external observer can see)
* **Invariants** (must always hold)
* **Failure semantics** (what happens when inputs/dependencies fail)
* Optional: **examples** (input/output examples, not implementation)

This pushes you towards specs that are testable without prescribing mechanics.

## 4) Independent tests from shared specs: good, but watch correlated failure

You already note the key limitation: both agents share the same ambiguous input.

Ways to strengthen independence without jumping straight to “holdout repo”:

* **Different representation**: derive tests from a *normalised* spec form (structured extraction) produced by a different pass than the code generator uses. If the normaliser and coder disagree, you surface it.
* **Adversarial test agent**: its explicit job is to find edge cases the coder will miss and to interpret ambiguity pessimistically. It’s still reading the same text, but incentives differ.
* **Metamorphic properties**: tests that assert relationships (“if you do X then Y must remain true”) rather than single examples; these catch lots of invented behaviour.
* **Coverage targets**: measurable expectations like “each MUST/SHALL statement yields ≥1 test” and “each error condition yields ≥1 test”.

Hidden/holdout tests are still valuable, but the above gives you earlier traction.

## 5) Unspec’d residue: important, but define “behaviour” carefully

If you try to flag any behaviour not traceable to a spec, you’ll quickly get swamped because:

* Libraries introduce behaviour.
* Error handling/logging/metrics create observable behaviour.
* Performance optimisations create timing differences.

So you need a taxonomy:

* **Runtime-exposed behaviour** (API responses, state transitions) — should be spec-traceable.
* **Operational behaviour** (logs, metrics, tracing) — may be governed by general “observability specs”.
* **Internal scaffolding** (helpers, adapters) — should be traceable *as supporting implementation* even if not directly spec’d.

A pragmatic approach is to make “residue detection” primarily operate on:

* Public interfaces (HTTP endpoints, CLI outputs, events/messages)
* Persistent storage schema and migrations
* Security-relevant flows (authz decisions, data egress points)

And treat internal code residue as a softer signal unless it’s clearly dead.

## 6) “Code is a real artefact” implies you need a notion of *state*

Kubernetes reconciliation works because you have:

* Desired state
* Observed state
* A controller that computes a delta

Here your “observed state” is:

* Code + tests + build artefacts + runtime config
* Plus whatever the system is currently doing in production (which you *don’t* fully observe)

If the reconciler only looks at repo state, it can still drift from production reality.

Even for a single-repo project, you’ll want explicit handling of:

* **Generated files** vs hand-authored files
* **Environment assumptions** (feature flags, config, secrets integration)
* **Backward compatibility contracts** (data migrations, API versioning)

This suggests a “context/constraint spec” category needs to include **deployment invariants** (what must remain compatible, what can break).

## 7) Brownfield “islands” is strong; the hard part is the seams

Where islands meet unmanaged code, you rely on interface specs. The dangerous part is *semantic mismatch* across boundaries (e.g., an unmanaged module has quirks that aren’t in the interface spec).

Two ideas:

* Treat interface specs as **executable contract tests** (consumer-driven contracts).
* Have a “seam hardening” mode: before expanding an island, invest in richer interface specs and tests at that boundary.

This keeps early adoption from turning into a false sense of control.

## 8) Refactoring: you probably need a spec concept for “quality”

You point out the refactoring question. If you don’t encode quality constraints, the reconciler will satisfy behaviour and accumulate complexity.

You can address this without pretending you can formally verify “clean code”:

* Define **structural constraints** that are mechanically checkable:

  * Module boundaries, dependency direction rules
  * Maximum layering violations
  * Banned imports
  * Cyclomatic complexity thresholds (rough, but measurable)
* Define **architecture conformance tests** (static analysis)
* Define **debt budgets** per subsystem and treat violations as reconciliation failures, not warnings

This gives Phase 2 something objective to optimise besides “make tests pass”.

## 9) A key missing piece: a normal form for specs

Markdown is great for humans, but reconciliation benefits from a canonical intermediate representation:

* Extracted entities, obligations (MUST/SHALL), invariants
* Inputs/outputs, error cases
* Cross-references resolved to stable IDs
* A “spec delta” between versions

This enables:

* Deterministic checks
* Better prompt assembly (less raw prose, more structured facts)
* Better traceability (“this function exists to satisfy obligation O-123”)

You can still keep markdown as source; the IR is derived.

## 10) Suggested discussion path (to make progress fast)

Pick one concrete slice and force it through the whole model mentally:

1. A small feature with a couple of edge cases and an integration point.
2. Write the specs in your proposed format.
3. Ask: what would the linter flag? what ambiguities remain?
4. Ask: what tests would the independent test agent derive?
5. Ask: what “residue” would appear even in a good implementation?
6. Ask: what would proportionality mean here?

If you choose the slice, I can do steps 2–6 in a structured way (and you’ll see where the spec format and the linter prompts need tightening).

## Concrete questions back to you (pick any to answer; no need to do all)

* What’s your target “first codebase”: a toy repo for this system itself, or applying it to an existing service?
* What language/toolchain do you expect Phase 2 to operate on first (Python/TS/Rust/etc.)?
* Do you envision specs expressing any concurrency/async semantics early on, or is that explicitly out of scope for v1?
* What’s the trust model: does Phase 2 ever auto-commit on main, or always via PR + CI?

If you answer only one: the intended first target codebase and language will constrain almost every other design choice.
