# ADR-003: PostgreSQL and Redis as Data Backbone

## Status

Accepted

## Context

The platform needs durable relational state plus fast ephemeral counters and caches.

## Decision

Use PostgreSQL for durable domain data and Redis for short-lived, high-frequency state such as quotas and rate limits.

## Consequences

- Positive: strong transactional guarantees with fast operational counters.
- Negative: two backing services to manage.

## Alternatives considered

- MongoDB-only approach: weaker relational integrity for billing and access control workflows.
- In-memory only counters: not viable for multi-instance correctness.
