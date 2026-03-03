---
Kind: feature
Specifies:
  - architecture.md#spec-parser-and-dag-builder
---

# Spec Parsing

Build a parser that reads spec files and constructs a navigable spec graph for downstream tooling.

## File Discovery

- Recursively read all `.md` files under the `specs/` directory.
- Ignore non-markdown files.
- Treat subdirectory structure as organisational only; parsing and graph semantics must not depend on directory depth.

## Link Extraction

- Parse front matter from each spec file.
- Extract `Kind`, `Specifies`, and optional `Root` metadata.
- Interpret each `Specifies` target as a directed edge from the current spec to the referenced section.

## DAG Construction

- Build graph nodes for each spec file and addressable heading section.
- Build graph edges from `Specifies` targets.
- Detect cycles and report the exact cycle path.

## Reference Resolution

- Resolve each `Specifies` target to an existing file and heading slug.
- Resolve cross-references in markdown links when they target local specs.
- Report unresolved targets with file and line context where available.

## Structural Error Reporting

- Report malformed front matter.
- Report invalid `Kind` values.
- Report malformed `Specifies` entries.
- Report invalid `Root` values and invalid `Root` placement on non-`feature` specs.
- Report when more than one spec declares `Root: true`.
- Continue parsing after recoverable errors and return all discovered issues in one pass.

## Parser Interface

- Expose parser behaviour as a library API for reuse by linter, reconciler, and traceability components.
- Also expose a CLI for checking that a spec directory can be successfully parsed.
