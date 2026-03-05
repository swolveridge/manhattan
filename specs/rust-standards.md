---
Kind: constraint
Specifies:
  - architecture.md#implementation-language
---

# Rust Standards

Apply these standards to all Rust code in this repository.

## Core Tooling

- Run `cargo fmt` and keep formatting clean.
- Run `cargo clippy --all-targets --all-features -D warnings`.
- Run `cargo test` for local verification before merge.
- Run `cargo check` in fast feedback loops.

## Initial Crate Guidance

- Use `anyhow` for application-level error aggregation.
- Use `thiserror` for typed library error definitions.
- Use `serde` with `serde_yaml` for front matter parsing and `serde_json` for JSON output surfaces.
- Use `pulldown-cmark` for markdown parsing and heading extraction.
- Use `regex` only for flat lexical pattern matching.
- Use dedicated parsers for structured or nested formats (for example markdown, JSON, YAML, or syntax trees).
- Use `clap` for CLI interfaces.
- Use `tokio` as the default async runtime where asynchronous execution is required.
- Use `tracing` and `tracing-subscriber` for structured logs.
- Use `petgraph` for DAG representation and cycle analysis.
- Use `insta` for snapshot tests where output contracts are textual and stable.

## Error Handling

- Return typed errors in library boundaries.
- Attach context to propagated errors.
- Avoid `unwrap` and `expect` in production paths.
- Fail fast on unrecoverable configuration errors.

## API and Module Design

- Keep parsing, graph, linting, prompting, and reconciliation in separate modules.
- Keep public APIs small and explicit.
- Prefer immutable data flow for parsed specs and graph structures.
- Hide implementation details behind clear interfaces.

## Testing

- Write unit tests for deterministic logic.
- Write integration tests for end-to-end workflows.
- Use `proptest` for property-based tests on parser, graph, and normalisation invariants.
- Where possible, prefer property-based tests using a reference implementation
  - Reference implementation must implement the same interface as real implementation
  - Property is `ref_impl(inputs) == real_impl(inputs)` 
- Keep fixtures small and purpose-specific.
- Tag tests with source spec references where practical.

## Performance and Safety

- Avoid unnecessary allocations in hot parsing and graph traversal paths.
- Avoid unsafe Rust unless a documented constraint spec permits it.
- Measure before optimising and document significant performance trade-offs.
