# ADR-001: Rust Axum for API Gateway

## Status

Accepted

## Context

The gateway must enforce authentication, input validation, and orchestration boundaries under concurrent load while keeping failure modes explicit.

## Decision

Use Rust with Axum for the external gateway service.

## Consequences

- Positive: strong compile-time guarantees, explicit error handling, low-overhead async runtime.
- Negative: higher implementation complexity and narrower developer familiarity.

## Alternatives considered

- Go Fiber: strong performance and simpler onboarding, but weaker type invariants.
- Node Fastify: rapid iteration, but runtime type safety depends more heavily on discipline.
