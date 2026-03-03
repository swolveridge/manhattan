# Spec-Driven Codebase Reconciliation: Design Exploration

## The Core Idea

A software system where **declarative specifications are the primary source of truth**, and a reconciliation process keeps the codebase aligned with them. Think Kubernetes-style desired-state reconciliation, but for application code rather than infrastructure.

Specs are markdown files committed to the same repository as the code. They describe what the system **does** (or should do), never phrased as imperative instructions. The system reads these specs and converges the codebase toward satisfying them.

---

## The Two-Phase Model

The workflow for any change — whether initiated by editing a spec or editing code — passes through two distinct phases.

### Phase 1: Spec Reconciliation (Interactive)

All specs across all levels are made consistent with each other. This is where human interaction lives. The system:

- Checks the spec corpus for contradictions, ambiguities, gaps, and staleness
- Derives or updates lower-level specs from higher-level ones
- Asks clarifying questions when intent is unclear
- Validates that cross-cutting concerns and interactions between features are addressed
- Potentially flags specs that may be hard to verify or implement reliably

This phase ends when the spec DAG is internally consistent and the human is satisfied.

**When triggered by a code change**: The system diffs the code change against existing specs, proposes spec updates, and enters the same clarification/consistency loop. The existing specs serve as strong priors about what the pre-change code did. Once specs are consistent, Phase 2 handles any downstream code changes. This means out-of-band code edits become holistically correct changes rather than point fixes that may not fit the whole system.

### Phase 2: Code Reconciliation (Autonomous)

The consistent spec DAG is the input. Code and tests are the output. No human in the loop. Trust comes from multi-layered verification:

- Independent test derivation (see below)
- Test execution
- Unspec'd residue detection
- Multi-agent collaboration with different priorities/personas

The result is a commit (or PR, depending on trust configuration) containing spec changes, code changes, and test changes, all traceable to each other.

---

## The Spec Corpus

### Structure

Specs are **markdown files** organized in folders within the repository. They form a **DAG** (directed acyclic graph) from high-level intent down to detailed behavioral specs. The DAG arrows point in one direction: a lower-level spec declares that it `specifies` a section of a higher-level document.

Addressing uses the natural path/to/file.md#heading convention. Cross-references are standard markdown links.

A lightweight convention at the top of each spec section declares its relationships:

```markdown
# Password Reset

Specifies: features/authentication.md#password-recovery

The system allows users to reset their password via email.
A reset token is single-use and expires after 30 minutes...
```

Cross-cutting concerns and integration points become visible when a spec has **multiple outgoing `specifies` links** to different higher-level documents. This is a signal that the spec describes an interaction between features, and the spec linter should pay particular attention to these.

### Spec Levels

Rather than strict layers, there are different **kinds** of spec, some of which lead to other kinds of spec (possibly multiple) rather than directly to code:

- **Intent / Feature specs**: High-level descriptions of what the system does for users
- **Behavioral specs**: Detailed descriptions of system behavior, edge cases, error handling
- **Interface / Contract specs**: API shapes, module boundaries, data schemas
- **Constraint / Context specs**: Facts about the real world the system operates in, architectural decisions, known limitations ("we tried X but it didn't work because of Z")

These are not ontologically different — constraints are essentially specs, perhaps with "the system must respect that..." prepended. They may be tagged or organized differently for query purposes, but they participate in the same DAG and the same consistency checking.

The **cut point** between where specs end and code begins: **spec the decisions, not the mechanics**. Anything involving a choice someone would want to review or could disagree with belongs in a spec. Anything that's a mechanical consequence of those decisions is code territory. This cut point can rise over time as models improve — you can spec at higher levels of abstraction and let the system derive more.

### Spec Duplication Across Levels

There is inherent duplication across spec levels ("users can log in" at the top, "authenticate via bcrypt with cost factor 12" at the bottom). This is **refinement, not duplication** — each level adds information. The risk is divergence when one level is updated without the others, but that's precisely what Phase 1 catches. Because specs are concise markdown rather than thousands of lines of code, LLMs are well-suited to detecting this kind of inconsistency.

The multi-level structure provides **readability at multiple zoom levels**: a new team member reads high-level specs to understand the system, an implementer drills into detailed specs for their area, a reviewer checks whether detailed specs are faithful to high-level intent.

---

## Tests as Evidence of Compliance

Tests are not specs. They are a **reconciliation output** — evidence that the code satisfies the specs. The key insight:

- A **separate agent** (from the one writing code) derives tests from the same specs
- This is functionally like having a different model do a code review by deriving tests and requiring the coding model's output to pass them
- The test derivation agent and the coding agent are independent of each other, though they share the spec as input

### Limitations of This Approach

Both agents read the same spec. If the spec is ambiguous in a way both agents interpret identically, tests will pass despite not matching the human's actual intent. The independence is from each other, not from the shared input.

**Holdout tests** (hidden from the coding agent, possibly in a separate repo or encrypted) remain a potentially valuable additional verification layer. We chose not to make them a primary mechanism because the multi-layered spec approach may reduce the need, but they shouldn't be discounted. They're most useful when they test **implications** of specs that aren't explicitly stated.

### Integration and End-to-End Tests

These are **crucial**. LLMs have a demonstrated tendency toward local correctness but global incoherence. Integration tests are a safety net against cross-feature emergent bugs that no individual spec covers. End-to-end tests verify that the system works as a whole, not just as a collection of individually correct components.

---

## Managing Code Growth and Minimality

LLMs have a strong prior toward adding code and a weak prior toward removing it. Running reconciliation loops multiple times tends to produce more and more code (observed empirically by a colleague building a similar system).

### Minimal Unspec'd Residue

The primary mechanism for combating this: **every behavior in the codebase should be traceable to a spec**. Code that exists but isn't traceable to any spec is flagged as "unspec'd residue" — it's either dead code, implicit behavior that needs a spec, or something the agent hallucinated.

This should extend beyond just lines of code to **behaviors**: if you can write a test demonstrating some behavior, there should be a spec that defines or at least implies that behavior should exist. This is behavioral spec coverage.

### Multi-Agent Inner Loop

The reconciliation process uses multiple agents with different priorities — at minimum a coder and a reviewer, potentially also an architect (thinking about how changes fit the bigger picture), a performance-focused agent, and others. These collaborate like a real team, or like a single developer wearing different hats.

These agent personas map naturally to the spec categories: the architect owns structural/cross-cutting specs, the performance agent owns performance constraints, the coder translates lowest-level specs to code, the reviewer checks compliance.

### Change Proportionality

There should be pushback against changing too much code in response to a small spec change. Sometimes large changes are necessary, but disproportionate changes should be flagged and justified. Existing code should be treated as a **strong prior** — make minimal changes to achieve the desired end state, which is how a human developer would generally approach it.

---

## Spec-to-Code Traceability

Knowing which code relates to which specs is essential for scoping reconciliation and detecting unspec'd residue. This is a many-to-many relationship (a spec touches multiple files, a function may serve multiple specs).

### Day-One Approach: Derived on Demand

Ask an LLM "which code implements this spec?" each time you need the mapping. Expensive in tokens but always fresh, and appropriate while the codebase is relatively small. Can be cached and invalidated when files change.

### Future Approaches

- **Manifest file**: A maintained traceability matrix, non-invasive but can drift. Rebuildable from scratch as a consistency check.
- **Structural alignment**: Convention that spec organization roughly mirrors code organization, reducing the search space.
- **Exploration agents**: For larger codebases, lightweight agents that explore and report summaries (as current tools like Claude Code already do effectively).

The system should not **depend** on any cached traceability being correct — it should always be rebuildable.

---

## Adoption and the Islands Model

### .specignore

Code excluded from reconciliation. The agent framework itself (e.g., Goose or similar) lives here. Conceptually, ignored code is no different from any third-party dependency — you don't spec `node_modules/`. The boundary between managed and unmanaged code is defined by **interface specs** that describe what the unmanaged code provides.

### Brownfield Adoption via Islands

Rather than a single expanding boundary, you can have **multiple islands** of spec-managed code within a larger unmanaged codebase:

1. Draw a boundary around a subsystem
2. Write specs for it (possibly with LLM assistance to draft from existing code)
3. Establish traceability for that subsystem
4. Bring it under reconciliation management
5. Everything outside is "unmanaged dependency with interface specs"
6. Gradually expand each island

Islands can grow independently. When two islands grow toward each other, their interface specs start describing the same boundary from opposite sides — that's the signal they can merge. Different teams can independently start islands. Adoption is inherently incremental, not a big-bang decision.

### Manual Code Within a Managed System

For specific algorithmic code or anything you want to maintain by hand, put it in a file and `.specignore` it. It becomes an unmanaged dependency with its own interface spec. The system can verify it satisfies relevant specs but won't try to rewrite it.

---

## The Spec Linter / Consistency Checker

A pre-reconciliation analysis pass on the spec corpus, running before any code is touched. This is potentially one of the most valuable pieces of the system. It checks for:

- **Contradictions**: Two specs that can't both be true
- **Ambiguity**: Specs with multiple plausible interpretations
- **Gaps**: Specs referencing undefined concepts or entities
- **Staleness**: Specs referencing superseded behaviors
- **Scope creep**: Spec changes with unaddressed implications for other specs
- **Implementability**: Flagging specs that may be hard to verify or implement reliably with current model capabilities
- **Completeness**: Whether specs are sufficiently detailed to produce correct code

This is a tractable LLM task — reasoning about concise declarative statements and their logical relationships, not producing working code. And it catches problems when they're cheap to fix (editing a spec) rather than expensive (debugging generated code satisfying contradictory requirements).

---

## Code as Artifact, Not Derivative

Since code generation is non-deterministic, code is a **real artifact** that must be committed and versioned. You can't reliably regenerate it. This means:

- Code is the ground truth of "what we're actually running"
- Specs are the ground truth of "what we intended"
- The system maintains alignment between these **two sources of truth**

This is more like **database replication** than compilation. Two representations of the same system, kept in consistency by a reconciliation process that can (eventually) work in both directions.

The reconciler should not run if nothing has changed — trust the last loop. But a **dry-run/sanity-check mode** should exist to verify specs are implemented without making changes.

---

## Open Questions and Uncertainties

### Semantic Verification Gap

The system depends on the ability to determine "does this code satisfy this spec?" LLMs are improving at this but not yet reliable for:

- Security properties (absence of information leakage, timing attacks)
- Race conditions and concurrency issues
- Subtle data corruption
- Performance under specific load patterns

These are properties where the multi-layered verification may not be sufficient with current models. The "fully autonomous Phase 2" may need human review gates for certain categories of spec for some time.

### Cross-Feature Emergent Behavior

Real bugs often emerge from the interaction of independently correct components. No individual spec covers "the system deadlocks when password reset and account deletion happen simultaneously." The spec linter can catch some interactions (specs with multiple `specifies` links), but the dangerous cases are interactions nobody anticipated. Integration testing helps but doesn't fully solve this.

### Refactoring

When code satisfies all specs but has become tangled or poorly structured, who initiates refactoring? The specs haven't changed. Code-quality properties (maintainability, coupling, readability) don't have obvious spec-level representations. Architectural specs ("follows hexagonal architecture") partially address this but are hard to verify automatically. The question of how much humans should interact with code directly remains open.

### Context Window vs. System Size

Exploration agents reporting summaries work for understanding large codebases. But reconciliation requires modifying code while holding enough context to avoid breaking things, and summaries are lossy. Whether this scales to large codebases is an empirical question we can't fully predict.

### Bootstrapping

First-time spec writing will inevitably miss implicit behaviors ("API returns dates in UTC," "whitespace is stripped from email inputs"). This means early unspec'd residue metrics will be noisy — high, but mostly legitimate. This is a potentially discouraging adoption experience.

Possible mitigations:
- Artificially high cut-over level (more detailed specs) at the start
- Multiple Phase 1 loops to extract implicit knowledge from humans
- LLM-assisted spec drafting from existing code

This requires its own dedicated discussion.

### Spec Evolution

When specs change, the reconciliation should just handle it. No explicit migration concept needed — the models can be given the diff as well as the new spec as context. The old spec state is captured in git history.

### Rationale and Audit Trail

Phase 2 code changes can be explained as "changed because it didn't match the spec," but this may be insufficient for debugging. Some mechanism for recording implementation rationale may be valuable, though this is another LLM output that might itself be unreliable. The right level of rationale capture is unclear.

---

## Precedents and Influences

- **Kubernetes**: Declarative desired state → reconciliation loop. The inspiration for the core model, though the semantic gap between infra and application code is enormous.
- **Terraform/Pulumi**: State files as "last known reality." The traceability manifest serves a similar function.
- **Database migrations**: The insight that transition plans matter, not just desired state. Though we currently lean toward "reconciliation handles it" rather than explicit migration planning.
- **Literate programming**: Documentation-first development. Failed historically because sync was too expensive. LLMs may change the economics.
- **Design-by-Contract**: Formal pre/postconditions as verifiable specs.
- **Architectural Decision Records**: The model for constraint/context specs that capture rationale and prevent repeated exploration of rejected approaches.
- **DO-178C / Systems Engineering**: Bidirectional traceability from requirements to code to tests. The spec coverage and traceability manifest ideas draw from this tradition.

---

## Summary of Key Design Decisions

| Decision | Rationale |
|---|---|
| Specs are declarative, never imperative | Idempotent interpretation; can be read at any time to understand desired state |
| Specs are markdown in the repo | Readable, diffable, auditable, editable with any tool |
| Spec DAG with arrows pointing upward (lower specs reference higher) | Cross-cutting concerns become visible as specs with multiple upward links |
| Two phases with human interaction only in Phase 1 | Clean trust boundary; ambiguity resolved before code generation |
| Tests derived independently from specs | Breaks circularity of agent testing its own code |
| Constraints are just specs | Simpler ontology; tag for query purposes but don't over-engineer the taxonomy |
| Code is a real artifact, not ephemeral | Non-deterministic generation means it must be versioned |
| Islands model for adoption | Incremental, non-disruptive, supports multiple independent efforts |
| Traceability derived on demand initially | Appropriate for early/small codebases; can add caching later |
| Unspec'd residue as a metric | Structural pushback against code growth, not just aesthetic preference |
| Holdout tests kept as a future option | May not be needed if multi-layer verification is sufficient, but don't discount |