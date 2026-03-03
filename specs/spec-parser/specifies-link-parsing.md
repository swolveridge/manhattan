---
Kind: behavioural
Specifies:
  - spec-parser/spec-parsing.md#link-extraction
---

# Specifies Link Parsing

Implement deterministic parsing rules for `Specifies` metadata in front matter.

## Accepted Shapes

- Accept `Specifies` as a YAML list of strings.
- Accept one or more targets in the list.
- Treat each list item as exactly one target.

## Target Format

- Require each target to match `relative/path/to/file.md#heading-slug`.
- Require file paths to be relative to `specs/` and reference markdown files in the corpus.
- Require heading slugs to be non-empty.

## Normalisation

- Preserve target strings exactly as written for reporting.
- Normalise heading comparisons using GitHub-flavoured markdown slug rules during resolution.

## Error Handling

- Report non-list `Specifies` values as invalid.
- Report non-string list items as invalid.
- Report malformed target strings as invalid.
- Continue parsing remaining files and return all link-parsing errors in one result.
