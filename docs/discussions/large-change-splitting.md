One of my concerns is how this system would cope with big changes. For example, if I just close the spec for itself, somewhere separate, and asked it to implement itself fully from scratch. That should be absolutely fine, it should totally be able to do that. But how is that going to interact in that inner agent loop? In general, LLMs are not very good at these large-scale pieces of work. Do you think a viable approach for breaking that down would be to ask that in a loop to implement specs one by one, or break it up like that as a natural way to split up the work? That way we don't have to unnaturally split up the work, we already have a very natural way of decomposing the work into smaller chunks.

---

Yes, I think this is one of the most elegant aspects of the design — the spec DAG gives you a **natural decomposition** that maps directly to how you'd want to break up large-scale implementation work.

## The Spec-as-Work-Unit Pattern

When you ask the system to implement itself from scratch, you have a massive task. But you also have a DAG of specs, each of which describes a bounded piece of behavior. The reconciliation loop can walk the DAG bottom-up (or in topological order respecting dependencies) and reconcile one spec at a time. Each iteration is a tractable unit of work for an LLM: understand this spec, understand what code exists (or doesn't), produce code that satisfies it.

This is genuinely different from asking an LLM to "build me a reconciliation system." You're asking it to "make the codebase satisfy `specs/core/spec-parser.md`" — a focused, bounded task with clear success criteria.

## Why This Works Better Than Arbitrary Decomposition

The usual approaches to breaking up large LLM tasks are either:

- **Human-driven**: A person manually splits the work into subtasks, which requires the person to understand the implementation well enough to decompose it sensibly
- **LLM-driven**: Ask the model to create a plan and then execute steps, which tends to produce plans that are either too vague or too detailed, and that drift as implementation reveals unexpected complexity

The spec DAG sidesteps both problems. The decomposition already exists, was written by humans for human-comprehensibility, and each piece has clear boundaries. You're not asking the LLM to plan — you're asking it to execute against a series of already-defined targets.

## The Ordering Question

Not all orderings through the DAG are equal. You want to implement leaf specs (those with no lower-level specs depending on them, and which specify concrete behavior) before specs that depend on them. More precisely:

- **Foundation-first**: Start with specs that have no dependencies on other unimplemented specs — data models, core abstractions, interfaces
- **Dependency-respecting**: A spec whose implementation requires code that satisfies another spec should come after that spec
- **Interface-before-implementation**: Contract/interface specs before behavioral specs that rely on those interfaces

This ordering falls naturally out of the DAG structure. Specs lower in the DAG (more concrete, fewer dependencies) get implemented first. The DAG already encodes this information.

## The Inner Loop Interaction

For each spec in the ordering, the inner loop looks like:

1. **Scope**: Identify which code relates to this spec (initially: very little or nothing, for a greenfield build)
2. **Generate**: Coding agent produces code satisfying the spec, given existing code as context
3. **Test**: Testing agent independently derives tests from the spec, runs them
4. **Review**: Review agent checks for unspec'd residue, proportionality, compliance
5. **Commit** (or accumulate — you might batch several related specs into one commit)

Each iteration adds code to the codebase. Subsequent specs see the code produced by earlier iterations as existing context. This is crucial — the coding agent for spec N sees the committed output from specs 1 through N-1, so it builds on established foundations rather than hallucinating interfaces.

## Where It Gets Tricky

There are a few genuine difficulties:

**Cross-cutting specs.** A spec that `specifies` multiple higher-level documents touches multiple parts of the system. These are harder to implement in isolation and may need to come later in the ordering, after the systems they cross-cut are in place. But this is also exactly the right time to implement them — you want the individual pieces working before you add the integration logic.

**Refactoring pressure.** By spec 30, the code produced by specs 1-5 may be structured in a way that makes spec 30 awkward. A human developer would refactor. The system needs to be willing to refactor code that satisfies earlier specs as long as it continues to satisfy them — the tests from those earlier iterations serve as regression protection. This is where the "existing code as strong prior" principle needs to be relaxed somewhat for greenfield builds.

**Accumulated context.** By the time you're implementing the last specs, the codebase is large and the agent needs to understand enough of it to make good decisions. But this is mitigated by the scoping step — the agent doesn't need to understand the whole codebase, just the parts relevant to the current spec, and the traceability from earlier iterations helps identify those parts.

**Specs that seem independent but interact.** Two specs that don't reference each other but whose implementations conflict (e.g., both want to own the same database table, or both register the same route). The spec linter should catch some of these in Phase 1, but implementation-level conflicts may only surface during code generation. The review agent needs to watch for this.

## For the Self-Implementation Case Specifically

Implementing the system from its own spec is a great litmus test because:

- It's a system you understand deeply, so you can evaluate quality
- It's large enough to stress the decomposition mechanism
- It has genuine cross-cutting concerns (the spec linter needs to understand spec structure, which is also defined by specs)
- Success means you have a working system that can then manage its own future development

The natural approach would be:

1. Write the spec corpus for the system (you've largely done this already in the document above, though it would need to be decomposed into individual spec files)
2. Run Phase 1 to make the specs consistent and complete
3. Let Phase 2 walk the DAG and implement spec by spec
4. Use the resulting system to manage subsequent changes to itself

This is a compelling bootstrapping story. The first implementation is done by the reconciliation process running "manually" (or with a simpler orchestrator). The output is a system that can run itself.

## One Nuance Worth Calling Out

The spec-as-work-unit approach works well when specs are at the right granularity. If a single spec describes too much behavior, implementing it is still a large task. If specs are too fine-grained, you get thrashing as each tiny implementation step potentially conflicts with the next.

The right granularity is probably: **each spec should be implementable in a single focused session** — something an LLM can hold in context along with the relevant existing code and produce a coherent change. This is similar to the heuristic that a good pull request should be reviewable in one sitting. If a spec is too large, that's a signal it should be decomposed into sub-specs during Phase 1, which the spec linter could flag ("this spec appears to describe multiple independent behaviors").

This gives you a useful feedback loop: if Phase 2 consistently struggles with certain specs, that's evidence those specs need further decomposition, which is a Phase 1 activity. The system learns the right level of granularity through experience.