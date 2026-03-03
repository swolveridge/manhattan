# The Build Plan (stage 5)

## Stage 5: Reconciler

**What we're doing:** Building the core of Phase 2 — the component that takes consistent specs and produces/updates code. This is the big one.

**Why last (among the components):** It's the most complex, benefits from all other components existing (it uses the parser, prompt generator, and test deriver), and mistakes in the reconciler are the most dangerous (it modifies code). Building it last means we have the most experience with the spec format, the best prompt templates, and a working test deriver to verify its output.

**Specs to write:**

### `specs/reconciliation.md`
`Kind: feature`
`Specifies: architecture.md#reconciliation`

High-level feature spec:
- The system reads consistent specs and produces code changes to align the codebase
- Phase 2 is autonomous — no human in the loop
- Output is a set of file changes (creates, updates, deletes)
- Verification is multi-layered: independently derived tests, unspec'd residue detection, change proportionality checks

### `specs/code-generation.md`
`Kind: behavioral`
`Specifies: reconciliation.md#code-generation`

How code is generated/updated:
- Given a spec and existing code (if any), produce updated code that satisfies the spec
- Existing code is a strong prior — minimize changes
- Generated code should be idiomatic for the project's language and style
- The reconciler sees: the target spec, related specs (from the DAG), existing code for the target, interface specs for dependencies

### `specs/change-proportionality.md`
`Kind: behavioral`
`Specifies: reconciliation.md#change-proportionality`

Guarding against over-generation:
- Small spec changes should produce small code changes (usually)
- Large code changes from small spec changes are flagged for review
- "Large" is measured in both lines changed and files touched
- The flag doesn't block the change, but it's surfaced in the output

### `specs/unspecd-residue-detection.md`
`Kind: behavioral`
`Specifies: reconciliation.md#residue-detection`

Finding code that shouldn't exist:
- After reconciliation, analyze the codebase for code not traceable to any spec
- Report unspec'd code with suggestions: is it dead code? Does it need a spec? Is it an implicit behavior?
- This is a separate analysis pass after code generation, not part of generation itself

### `specs/multi-agent-loop.md`
`Kind: behavioral`
`Specifies: reconciliation.md#multi-agent`

The inner collaboration loop:
- At minimum: a coder agent and a reviewer agent
- The coder generates/updates code from specs
- The reviewer checks the code against specs independently
- Disagreements are resolved by re-examining the spec (not by compromise)
- Additional personas (architect, performance) are future extensions
- Each agent gets its own prompt, assembled by the prompt generator

### `specs/reconciliation-workflow.md`
`Kind: behavioral`
`Specifies: reconciliation.md#workflow`

The end-to-end reconciliation flow:
1. Identify which specs have changed (or which code is out of alignment)
2. Use traceability to scope the affected code
3. For each affected area: generate/update code (coder agent)
4. Review generated code (reviewer agent)
5. Derive tests independently (test deriver)
6. Run tests
7. If tests fail, iterate (coder revises, re-test)
8. Run unspec'd residue detection
9. Report results: changes made, tests passed/failed, residue found, proportionality flags

### Reconciliation Write Boundary

- The reconciler must not change files or directories covered by `.specignore`.

### `specs/reconciler-cli.md`
`Kind: interface`
`Specifies: reconciliation.md`

CLI interface:
- `spec-reconcile run` — full reconciliation: find misaligned code, fix it
- `spec-reconcile run --spec path/to/spec.md` — reconcile only code related to this spec
- `spec-reconcile dry-run` — report what would change without making changes
- `spec-reconcile check` — verify current code satisfies specs (no changes, just report)
- Output: summary of changes, test results, residue report, proportionality flags
- `--auto-commit` flag to commit changes directly vs. leaving them as working tree changes

**What to tell the coding agent:**

This is too big for a single prompt. Break it into sessions:

**Session 1: Core code generation**
```
I'm building a spec management system. Here's the full context:
[paste architecture]
[paste reconciler feature spec and code-generation behavioral spec]
[point to existing components: parser, linter, prompt generator, test deriver]

Build the core reconciliation logic: given a spec and existing code, 
produce updated code. Use the prompt generator for LLM calls. This 
is the simplest version — single agent, no review loop yet.

Include the reconciliation prompt template, following prompt standards.
```

**Session 2: Multi-agent loop**
```
Here's the current reconciler implementation:
[point to code from session 1]

Now add the multi-agent inner loop per this spec:
[paste multi-agent-loop behavioral spec]

The reviewer agent should have its own prompt template that checks 
code against specs independently from the coder.
```

**Session 3: Verification integration**
```
Here's the current reconciler:
[point to code from sessions 1-2]

Now integrate the test deriver and add the verification steps 
per this spec:
[paste reconciliation-workflow behavioral spec]

After code generation + review, the reconciler should:
- Call the test deriver to generate tests from the spec
- Run the tests against the generated code
- If tests fail, send failures back to the coder for revision
- Report results
```

**Session 4: Residue detection and proportionality**
```
Here's the current reconciler:
[point to code from sessions 1-3]

Add unspec'd residue detection and change proportionality checking:
[paste both behavioral specs]

These are analysis passes that run after code generation and 
report findings. They don't block changes, they surface information.
```

**Session 5: CLI and integration**
```
Here's the full reconciler:
[point to all code]

Build the CLI interface:
[paste CLI spec]

And ensure everything works end to end. Include integration tests 
that: take a spec, reconcile code, run derived tests, and verify 
the whole pipeline works.
```

**How to validate:**

This needs careful, graduated testing:

1. **Unit test the components:** Does the coder agent produce reasonable code from a simple spec? Does the reviewer agent catch obvious spec violations?

2. **Test on the existing codebase:** Run `spec-reconcile check` on the system itself. It should report that existing code (built by hand with Claude) mostly satisfies the specs. Discrepancies reveal either code bugs or spec ambiguities — both valuable to find.

3. **Test a real change:** Modify a spec for the linter (add a new check type, for example). Run `spec-reconcile run --spec specs/spec-linting.md`. Does it produce a reasonable code change? Do the independently derived tests pass?

4. **Test proportionality:** Make a tiny spec change. Does the reconciler make a proportionate code change, or does it rewrite everything?

5. **Test residue detection:** Add some dead code manually. Does residue detection flag it?
