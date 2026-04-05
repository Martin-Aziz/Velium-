# ADR-004: Next.js App Router for Frontend

## Status

Accepted

## Context

The product needs mixed rendering modes: server-rendered dashboard data and highly interactive client-side generation workspace.

## Decision

Use Next.js App Router with server components where appropriate and client components for generation interactions.

## Consequences

- Positive: performant initial loads and clear data-fetch boundaries.
- Negative: steeper conceptual model for server/client component boundaries.

## Alternatives considered

- Traditional SPA only: simpler model but slower first load and larger client bundles.
