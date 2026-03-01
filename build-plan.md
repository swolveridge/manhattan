# The Build Plan

## What We're Building

A system that keeps code aligned with declarative specs, built using existing AI coding tools, guided by specs from day one. The system will eventually manage itself, but we don't force that prematurely.

## What Tools We're Using During Construction

Throughout this plan, "ask the coding agent" means using whatever agent you're comfortable with (Claude Code, Goose, Cursor, etc.) in its normal interactive mode. The key discipline is: **always give it the relevant specs as context**, not just ad-hoc instructions. This means the specs are being tested as communication artifacts from the start, even before any tooling processes them.

---

## Stage 0: Meta-Specs

**What we're doing:** Establishing the conventions everything else follows. No code.

**Artifacts to write (by hand, in your editor):**

### `specs/meta/spec-format.md`
The format convention. Covers:
- Specs are markdown files under `specs/`
- Directory organization by kind (`features/`, `behavioral/`, `interface/`, `constraints/`)
- The `Specifies: path/to/file.md#heading` convention for DAG edges
- Cross-references as standard markdown links
- Declarative voice ("the system does X" not "do X")
- What belongs in a spec vs. what's code territory

**Reasoning:** Everything downstream parses or writes specs. If the format is ambiguous, every tool will handle edge cases differently. Get this right first. But keep it short — one to two pages. You'll revise it as you learn what works.

### `specs/meta/prompt-standards.md`
How prompts used by the system should be structured. Covers:
- Role assignment at the start of each prompt
- Explicit constraints (what the LLM should not do)
- Output format requirements
- Context delimiters between instructions and provided content
- Single responsibility per prompt

**Reasoning:** Every component of the system sends prompts to LLMs. Consistent prompt structure makes them easier to debug, test, and improve. Also, this document becomes context for the coding agent when it's building prompt templates later — "follow these standards."

### `specs/meta/architecture.md`
High-level system architecture. Covers:
- The two-phase model (spec reconciliation → code reconciliation)
- Components and their relationships (parser, linter, prompt generator, test deriver, reconciler, traceability)
- What's in `.specignore` and why
- The principle of specs as source of truth, code as real artifact

**Reasoning:** This is the map. When you're asking a coding agent to build component X, it needs to understand where X fits in the whole system. This doc provides that context without requiring the agent to read the entire spec corpus.

### `specs/meta/bootstrapping.md`
A constraint/context spec capturing the build plan (essentially a refined version of this document). Why we're building in this order, what we tried and rejected (self-bootstrapping), what the adoption model is.

**Reasoning:** ADR-style. Prevents re-litigating decisions. Also useful context for the coding agent — "we're building the linter, here's why it comes before the reconciler."

**How to validate Stage 0:** Read the meta-specs yourself. Do they make sense? Could a new team member read them and understand the project? Could a coding agent read them and understand what it's building? Show them to Claude and ask "what's ambiguous or missing here?" Not as a formal tool — just in conversation.

---

## Stage 1: Spec Parser / DAG Builder

**What we're doing:** Building the foundation that all other tools need — the ability to read spec files, extract their relationships, and build the DAG.

**Specs to write:**

### `specs/features/spec-parsing.md`
High-level feature spec:
- The system reads markdown spec files from a directory tree
- It extracts `Specifies:` declarations and builds a directed acyclic graph
- It resolves cross-references between specs
- It detects and reports structural problems (cycles, broken links, missing targets)

### `specs/behavioral/specifies-link-parsing.md`
`Specifies: features/spec-parsing.md#link-extraction`

Detailed behavior:
- `Specifies:` appears on the line immediately after a heading
- Multiple `Specifies:` links are allowed (one per line)
- The target format is `path/to/file.md#heading-slug`
- Heading slugs follow GitHub-flavored markdown conventions (lowercase, hyphens for spaces)
- How to handle malformed links (report error, continue parsing)

### `specs/behavioral/dag-construction.md`
`Specifies: features/spec-parsing.md#dag-construction`

Detailed behavior:
- Nodes are heading-level sections within spec files
- Edges point from the specifying node to the specified node (lower to higher)
- Cycle detection with clear error reporting (which nodes form the cycle)
- Orphan detection (specs that neither specify anything nor are specified by anything)

### `specs/interface/parser-api.md`
`Specifies: features/spec-parsing.md`

The parser's programmatic interface:
- Input: a directory path
- Output: a DAG structure (nodes with content, edges with source/target)
- Error reporting structure (list of issues with file/line references)
- This is a library, not a CLI (other tools will call it)

**What to tell the coding agent:**

```
I'm building a spec management system. Here's the architecture: 
[paste specs/meta/architecture.md]

I'm starting with the spec parser. Here are the specs for it:
[paste all four specs above]

Please implement this as a [Python/TypeScript/whatever] library. 
Follow the interface spec. Include unit tests that verify the 
behavioral specs.

The implementation should be straightforward — read markdown files, 
regex or simple parsing for the Specifies: convention, build a 
graph data structure, detect cycles with a standard algorithm.
```

**Reasoning for building this first:** The linter needs to traverse the spec DAG. The reconciler needs to find related specs. The traceability system needs to know which specs exist. Everything starts with parsing specs. And it's simple — well-understood algorithms (markdown parsing, graph construction, cycle detection), easy to test, low risk.

**How to validate:** Write a few test spec files with known relationships, including deliberate errors (cycles, broken links). Run the parser. Does the DAG match what you expect? Do the errors get caught? Also: **point the parser at your actual `specs/` directory.** It should parse the meta-specs and the parser's own specs and produce a valid (if small) DAG.

---

## Stage 2: Spec Linter

**What we're doing:** Building the consistency checker. This is Phase 1's workhorse — the tool that finds problems in specs before any code is written.

**Specs to write:**

### `specs/features/spec-linting.md`
High-level feature spec:
- The system analyzes a spec corpus for problems
- Categories of problems: structural, semantic, completeness
- Structural checks are deterministic (use the parser)
- Semantic checks use LLM analysis
- Output is a report with issues, severity, locations, and explanations

### `specs/behavioral/structural-checks.md`
`Specifies: features/spec-linting.md#structural-checks`

Deterministic checks that don't need an LLM:
- Broken `Specifies:` links (target file or heading doesn't exist)
- Cycles in the DAG
- Orphan specs (not connected to anything)
- Specs with `Specifies:` links to non-spec files
- Missing required sections (every spec should have at least a description)

### `specs/behavioral/contradiction-detection.md`
`Specifies: features/spec-linting.md#contradiction-detection`

LLM-powered analysis:
- Compare specs that specify the same parent (sibling specs)
- Compare specs across levels (does the detailed spec contradict the high-level one?)
- Distinguish contradictions from refinements (adding detail is fine, changing meaning isn't)
- Report with quotes from both specs showing the contradiction
- Confidence rating (high/medium/low)

### `specs/behavioral/gap-detection.md`
`Specifies: features/spec-linting.md#gap-detection`

LLM-powered analysis:
- Identify concepts referenced but never defined
- Identify edge cases mentioned at high level but not spec'd in detail
- Identify missing error handling specs (what happens when X fails?)
- Flag specs that reference external systems without interface specs

### `specs/behavioral/ambiguity-detection.md`
`Specifies: features/spec-linting.md#ambiguity-detection`

LLM-powered analysis:
- Identify specs with multiple plausible interpretations
- Identify vague quantifiers ("quickly," "efficiently," "appropriately")
- Identify undefined terms that could mean different things in different contexts

### `specs/interface/linter-cli.md`
`Specifies: features/spec-linting.md`

CLI interface:
- `spec-lint check [directory]` — run all checks, report issues
- `spec-lint check --structural-only` — skip LLM-powered checks (fast, cheap)
- `spec-lint check --focus path/to/spec.md` — check one spec and its relationships
- Output format: human-readable by default, JSON with `--json` flag
- Exit code: 0 for no issues, 1 for warnings, 2 for errors

**What to tell the coding agent:**

```
I'm building a spec management system. Here's the architecture:
[paste specs/meta/architecture.md]

I've already built the spec parser:
[paste specs for parser + point to the implementation]

Now I'm building the spec linter. Here are the specs:
[paste all linter specs]

And here are the prompt standards to follow for LLM-powered checks:
[paste specs/meta/prompt-standards.md]

Please implement this. The structural checks should use the parser 
library directly. The semantic checks (contradiction, gap, ambiguity) 
each need an LLM prompt — please create these as separate prompt 
template files following the prompt standards.

Include tests for the structural checks (deterministic, easy to test). 
For the semantic checks, include a few test cases with obviously 
contradictory/ambiguous specs and verify the linter catches them.
```

**Reasoning for building this second:** The linter is the first tool that provides value to *any* project, not just this one. It validates your spec-writing practice. And it's the core of Phase 1 — without it, spec reconciliation is entirely manual.

**How to validate:**

First, structural validation:
- Run `spec-lint check specs/` on the project's own specs
- Deliberately introduce a broken link, verify it's caught
- Deliberately introduce a cycle, verify it's caught

Then, semantic validation:
- Write two specs that subtly contradict each other. Does the linter catch it?
- Write a spec with an ambiguous term. Does the linter flag it?
- Write a spec that references "the notification system" when no notification spec exists. Does gap detection catch it?

**First real dogfooding moment:** The linter checking its own specs and the parser's specs. Fix any issues it finds. You're now using the system to maintain itself, albeit only the analysis part.

---

## Stage 3: Prompt Generator

**What we're doing:** Building the component that constructs prompts from specs and templates. The linter from Stage 2 already has some hand-crafted prompts. Now we formalize how prompts are built.

**Why now and not earlier:** In the self-bootstrapping plan, this was the kernel — the first thing built. But since we're using existing tools, we can build it after the linter. We already have working prompts (the linter's). Now we're extracting the pattern.

**Why now and not later:** The reconciler and test deriver both need structured prompts. Building the prompt generator now means those components get consistent, well-structured prompts. Also, the linter's prompts were probably somewhat ad-hoc — this is a chance to clean them up.

**Specs to write:**

### `specs/features/prompt-generation.md`
High-level feature spec:
- The system constructs LLM prompts from templates and context
- Templates are parameterized with spec content, code content, and metadata
- Prompts follow the standards in `specs/meta/prompt-standards.md`
- Output is a complete prompt string ready to send to an LLM API

### `specs/behavioral/template-filling.md`
`Specifies: features/prompt-generation.md#template-filling`

How templates work:
- Templates are text files with placeholder syntax (e.g., `{{spec_content}}`, `{{code_content}}`)
- Available variables: spec content, spec metadata (path, specifies links), code content, file paths, diff content
- Conditional sections (include this block only if code_content is provided)
- How to handle missing variables (error, not silent empty string)

### `specs/behavioral/context-assembly.md`
`Specifies: features/prompt-generation.md#context-assembly`

How context is gathered for a prompt:
- Given a spec, find related specs (parent, siblings, children in the DAG)
- Include relevant related specs as additional context
- Manage total context size (truncation strategy, prioritization)
- Delimiter conventions between sections of context

### `specs/interface/prompt-generator-api.md`
`Specifies: features/prompt-generation.md`

Programmatic interface:
- Input: template name, primary spec, optional code content, optional additional context
- Output: complete prompt string
- Template discovery: looks for templates in a `prompts/` directory
- Validation: check that all required template variables are provided

**What to tell the coding agent:**

```
I'm building a spec management system. Here's the architecture:
[paste specs/meta/architecture.md]

I've built the parser and linter:
[point to their implementations and specs]

The linter already has some prompt templates for its semantic checks. 
Now I'm extracting the prompt generation pattern into a dedicated 
component. Here are the specs:
[paste prompt generator specs]

Please implement this and then refactor the linter to use it. The 
linter's existing prompts should become templates in the prompts/ 
directory, filled using the new prompt generator.
```

**How to validate:** The linter should work exactly as before after the refactoring. Run `spec-lint check specs/` and verify identical results. Then inspect the extracted prompt templates — do they follow the prompt standards?

---

## Stage 4: Test Deriver

**What we're doing:** Building the independent agent that generates tests from specs. This is the verification mechanism that lets Phase 2 be autonomous.

**This is architecturally significant.** The test deriver must be independent from the code-writing component. It reads the same specs but produces tests without seeing the implementation. This is the "different model doing code review by writing tests" insight from the design doc.

**Specs to write:**

### `specs/features/test-derivation.md`
High-level feature spec:
- A separate agent reads specs and produces test cases
- Tests verify the behavioral claims made by specs
- The test agent does not see the implementation code
- Tests are executable (not just descriptions)
- Tests cover happy paths, edge cases, and error cases described in specs

### `specs/behavioral/test-independence.md`
`Specifies: features/test-derivation.md#independence`

The independence guarantee:
- The test derivation prompt includes spec content but never implementation code
- The test deriver may see interface specs (API shapes, function signatures) so tests can actually call something
- The test deriver does NOT see behavioral implementation details
- This separation is enforced by what context the prompt generator assembles, not by honor system

### `specs/behavioral/test-coverage-mapping.md`
`Specifies: features/test-derivation.md#coverage`

How specs map to tests:
- Each behavioral spec section should produce at least one test
- Edge cases and error conditions described in specs produce their own tests
- Tests are tagged with which spec section they verify (traceability)
- The deriver reports which spec sections it couldn't produce tests for (untestable specs)

### `specs/behavioral/integration-test-derivation.md`
`Specifies: features/test-derivation.md#integration-tests`

How cross-cutting tests are derived:
- Specs with multiple `Specifies:` links (cross-cutting concerns) trigger integration tests
- Integration tests verify that the interaction between features works, not just each feature in isolation
- The deriver is given all relevant specs for an integration point, not just one

### `specs/interface/test-deriver-api.md`
`Specifies: features/test-derivation.md`

Interface:
- Input: a spec (or set of specs), interface specs for the code under test
- Output: test files, tagged with source spec references
- CLI: `spec-test derive path/to/spec.md` — generate tests for a spec
- CLI: `spec-test derive --all` — generate tests for all specs
- CLI: `spec-test check` — run derived tests, report results per spec

**What to tell the coding agent:**

```
I'm building a spec management system. Here are the architecture 
and relevant existing components:
[paste architecture, parser specs, prompt generator specs]

Now I'm building the test deriver. Here are its specs:
[paste test deriver specs]

Key constraint: the test derivation prompts must NOT include 
implementation code. They should include the behavioral spec 
and the interface spec (so the tests know what functions/APIs 
to call), but nothing about how those functions are implemented.

Use the prompt generator to build the test derivation prompts. 
Follow the prompt standards.
```

**How to validate:**

This is the first really interesting validation step. You can:

1. Run `spec-test derive specs/behavioral/contradiction-detection.md` — this generates tests for the linter's contradiction detection, based only on the spec
2. Run those tests against the actual linter implementation
3. Do they pass? If not, either:
   - The linter implementation doesn't match the spec (fix the implementation or the spec)
   - The test deriver misunderstood the spec (fix the spec — it's ambiguous — or fix the test derivation prompts)
   - The test deriver generated bad tests (improve the test derivation prompts)

Each of these failure modes is informative. This is where you learn the most about whether the spec-driven approach actually works.

**Run the test deriver against all existing components' specs.** This is a comprehensive validation of everything built so far. Fix whatever breaks.

---

## Stage 5: Reconciler

**What we're doing:** Building the core of Phase 2 — the component that takes consistent specs and produces/updates code. This is the big one.

**Why last (among the components):** It's the most complex, benefits from all other components existing (it uses the parser, prompt generator, and test deriver), and mistakes in the reconciler are the most dangerous (it modifies code). Building it last means we have the most experience with the spec format, the best prompt templates, and a working test deriver to verify its output.

**Specs to write:**

### `specs/features/reconciliation.md`
High-level feature spec:
- The system reads consistent specs and produces code changes to align the codebase
- Phase 2 is autonomous — no human in the loop
- Output is a set of file changes (creates, updates, deletes)
- Verification is multi-layered: independently derived tests, unspec'd residue detection, change proportionality checks

### `specs/behavioral/code-generation.md`
`Specifies: features/reconciliation.md#code-generation`

How code is generated/updated:
- Given a spec and existing code (if any), produce updated code that satisfies the spec
- Existing code is a strong prior — minimize changes
- Generated code should be idiomatic for the project's language and style
- The reconciler sees: the target spec, related specs (from the DAG), existing code for the target, interface specs for dependencies

### `specs/behavioral/change-proportionality.md`
`Specifies: features/reconciliation.md#change-proportionality`

Guarding against over-generation:
- Small spec changes should produce small code changes (usually)
- Large code changes from small spec changes are flagged for review
- "Large" is measured in both lines changed and files touched
- The flag doesn't block the change, but it's surfaced in the output

### `specs/behavioral/unspecd-residue-detection.md`
`Specifies: features/reconciliation.md#residue-detection`

Finding code that shouldn't exist:
- After reconciliation, analyze the codebase for code not traceable to any spec
- Report unspec'd code with suggestions: is it dead code? Does it need a spec? Is it an implicit behavior?
- This is a separate analysis pass after code generation, not part of generation itself

### `specs/behavioral/multi-agent-loop.md`
`Specifies: features/reconciliation.md#multi-agent`

The inner collaboration loop:
- At minimum: a coder agent and a reviewer agent
- The coder generates/updates code from specs
- The reviewer checks the code against specs independently
- Disagreements are resolved by re-examining the spec (not by compromise)
- Additional personas (architect, performance) are future extensions
- Each agent gets its own prompt, assembled by the prompt generator

### `specs/behavioral/reconciliation-workflow.md`
`Specifies: features/reconciliation.md#workflow`

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

### `specs/interface/reconciler-cli.md`
`Specifies: features/reconciliation.md`

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

3. **Test a real change:** Modify a spec for the linter (add a new check type, for example). Run `spec-reconcile run --spec specs/features/spec-linting.md`. Does it produce a reasonable code change? Do the independently derived tests pass?

4. **Test proportionality:** Make a tiny spec change. Does the reconciler make a proportionate code change, or does it rewrite everything?

5. **Test residue detection:** Add some dead code manually. Does residue detection flag it?

---

## Stage 6: Traceability

**What we're doing:** Building the mapping between specs and code. Placed last because at small codebase sizes you can get by without it (just ask the LLM), but the reconciler's residue detection and change scoping work better with it.

**Specs to write:**

### `specs/features/traceability.md`
High-level feature spec:
- The system can determine which code implements which specs
- The relationship is many-to-many
- Traceability is derived on demand (ask an LLM)
- Results can be cached but the cache is never trusted as authoritative
- The cache can be rebuilt from scratch as a consistency check

### `specs/behavioral/derived-traceability.md`
`Specifies: features/traceability.md#on-demand-derivation`

How traceability is determined:
- Given a spec, find code files that implement it
- Given a code file, find specs it implements
- The LLM receives the spec content and a summary of available code files
- For large codebases, use exploration (file listing, grep, targeted reads) before detailed analysis
- Report confidence levels (high/medium/low)

### `specs/behavioral/residue-integration.md`
`Specifies: features/traceability.md#residue-detection`

Integration with unspec'd residue detection:
- Code that doesn't trace to any spec is candidate residue
- `.specignore` files and directories are excluded from residue analysis
- Traceability for dependencies: code that implements an interface spec for an external dependency is not residue

### `specs/interface/traceability-api.md`
`Specifies: features/traceability.md`

Interface:
- `spec-trace spec path/to/spec.md` — show which code implements this spec
- `spec-trace code path/to/code.py` — show which specs this code implements
- `spec-trace orphans` — show code files that don't trace to any spec
- `spec-trace matrix` — show the full traceability matrix
- `--rebuild-cache` flag to force fresh derivation

**What to tell the coding agent:**

```
I'm building a spec management system. Here's the context:
[paste architecture, point to existing components]

Build the traceability system:
[paste traceability specs]

This should integrate with the reconciler's residue detection — 
the reconciler should use traceability queries to scope its work 
and to find unspec'd code.
```

**How to validate:** Run `spec-trace matrix` on the system itself. Does the mapping look right? Does it correctly identify which code files implement which specs? Run `spec-trace orphans` — are there code files that don't map to any spec? (There probably are — utility files, configuration, etc. These either need specs or `.specignore` entries.)

---

## Stage 7: Close the Loop

**This is where the system starts managing itself.**

**What we're doing:** Using the reconciler on the system's own codebase. Carefully.

**Sequence:**

### 7a: Reconciler on the parser

The parser is the simplest, most stable component. Low risk.

```
spec-reconcile check --spec specs/features/spec-parsing.md
```

Does it report the existing parser code satisfies the specs? If there are discrepancies, understand why before proceeding.

Then make a small spec change to the parser. Something safe — maybe add a new structural check or change an error message format.

```
spec-reconcile run --spec specs/features/spec-parsing.md
```

Review the diff. Run the independently derived tests. Does it look right?

### 7b: Reconciler on the linter

Same process. Check, then small change, then review.

### 7c: Reconciler on the prompt generator

Same process.

### 7d: Reconciler on the test deriver

Getting more meta. The reconciler is now modifying the test deriver, whose output is used to verify the reconciler's work. Proceed carefully.

### 7e: Reconciler on itself

The final step. The reconciler modifying its own code.

**Safeguards for this step:**
- Always review diffs manually at this stage
- Keep a known-good version tagged in git so you can rollback
- Run the full test suite (not just the tests for the changed component) after each change
- Consider holdout tests for the reconciler — tests it can't see, written by hand, that verify critical properties

### 7f: Full self-management

Once you've done several successful self-reconciliation cycles with manual review, you can start trusting the system to make changes with less oversight. This is a judgment call, not a binary switch.

---

## Summary: What Gets Built When

```
Stage 0 │ Meta-specs           │ By hand        │ No code
Stage 1 │ Spec parser          │ Coding agent   │ Foundation
Stage 2 │ Spec linter          │ Coding agent   │ First useful tool
Stage 3 │ Prompt generator     │ Coding agent   │ Prompt infrastructure  
Stage 4 │ Test deriver         │ Coding agent   │ Verification capability
Stage 5 │ Reconciler           │ Coding agent   │ The core system
Stage 6 │ Traceability         │ Coding agent   │ Scoping and residue
Stage 7 │ Self-reconciliation  │ The system     │ Loop closed
```

At each stage from 1 onward, you're giving the coding agent:
1. The architecture spec (always)
2. The relevant component specs (always)
3. The prompt standards (when the component involves LLM calls)
4. Pointers to existing components it needs to integrate with
5. A clear statement of what to build in this session

At each stage, you validate by:
1. Running the new component
2. Running `spec-lint check` on the new specs (from Stage 2 onward)
3. Running the test deriver + derived tests (from Stage 4 onward)
4. Running `spec-reconcile check` to verify alignment (from Stage 5 onward)

The verification gets richer as you build more tools, which is why the order matters.

---

## What I'm Uncertain About

**Stage 5 session breakdown.** I split the reconciler into five coding sessions. That might be too granular or not granular enough — depends on the coding agent's context window and how cleanly the pieces separate. You might need to adjust based on what works.

**The multi-agent loop complexity.** The reconciler spec calls for coder + reviewer agents. Getting this right — especially the iteration when tests fail — is genuinely hard. It might make sense to start with a single-agent reconciler and add the reviewer as a second pass once the basic flow works.

**Test deriver quality.** The whole verification story depends on the test deriver producing good tests. If it produces shallow tests, the reconciler's output won't be well-verified. This is the component most likely to need iterative improvement of its prompts.

**Stage 7 ordering.** I ordered the self-reconciliation from simplest to most complex component. An alternative is to order by risk — start with the component where a reconciler mistake does the least damage. These might not be the same ordering.