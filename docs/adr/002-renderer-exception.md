# ADR-002: Node Renderer Exception in Polyglot Backend

## Status

Accepted

## Context

High-quality `.docx` rendering requires mature tooling with reliable heading, paragraph, and list handling.

## Decision

Use a dedicated Node.js TypeScript renderer service with the `docx` package, while keeping gateway and orchestration logic in Rust.

## Consequences

- Positive: practical document quality and easier extension for additional formats.
- Negative: one additional runtime and operational surface area.

## Alternatives considered

- Rust-native docx crates: not yet mature enough for current quality bar.
- Pandoc shell pipeline: increases process overhead and operational complexity.
