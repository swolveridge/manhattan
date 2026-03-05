---
Kind: interface
Specifies:
  - spec-linter/linter-cli.md#diagnostics-contract
---

# Linter Issue Codes

Define the stable issue code set emitted by the linter.

## Semantic Issue Codes

- `SEM_CONTRADICTION`: contradiction findings from semantic contradiction detection.
- `SEM_GAP`: gap findings from semantic gap detection.
- `SEM_AMBIGUITY`: ambiguity findings from semantic ambiguity detection.

## Structural Policy Issue Codes

- `STRUCT_ROOT_MULTIPLE`: more than one spec declares `Root: true`.
- `STRUCT_ROOT_KIND`: a non-`feature` spec declares `Root: true`.
- `STRUCT_MISSING_HEADING`: a spec has no markdown headings.
- `STRUCT_ORPHAN_SPEC`: an in-scope spec requiring parent links has no valid `Specifies` targets.

## Parser-Derived Structural Issue Codes

- `STRUCT_IOREADFAILURE`
- `STRUCT_INVALIDFRONTMATTER`
- `STRUCT_INVALIDKIND`
- `STRUCT_INVALIDROOT`
- `STRUCT_INVALIDSPECIFIES`
- `STRUCT_INVALIDSPECIFIESTARGET`
- `STRUCT_INVALIDCROSSREFERENCE`
- `STRUCT_BROKENTARGETFILE`
- `STRUCT_BROKENTARGETHEADING`
- `STRUCT_BROKENCROSSREFERENCEFILE`
- `STRUCT_BROKENCROSSREFERENCEHEADING`
- `STRUCT_CYCLEDETECTED`
