---
Kind: behavioural
Specifies:
  - spec-parser/spec-parsing.md#dag-construction
---

# DAG Construction

Construct and validate the directed acyclic graph derived from spec metadata.

## Node Model

- Create a file-level node for every parsed spec file.
- Create section-level nodes for every markdown heading in every parsed spec file.
- Link section nodes to their containing file node for traversal and diagnostics.

## Edge Model

- Create a directed edge from the source spec file to each target section in `Specifies`.
- Retain multiple outgoing edges for cross-cutting specs.

## Cycle Detection

- Detect directed cycles across file and section nodes.
- Report each detected cycle with an ordered list of node addresses that form the loop.

## Broken Target Detection

- Flag targets whose file does not exist.
- Flag targets whose heading slug does not exist in the target file.
- Keep broken-target diagnostics separate from orphan diagnostics.
