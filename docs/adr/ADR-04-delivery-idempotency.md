# ADR-04: Delivery Semantics and Idempotency Model

- Status: Accepted
- Priority: P0
- Date: 2026-05-09

## Context

Kafka and distributed consumers naturally operate with at-least-once delivery.  
Trading and money workflows must avoid duplicate side effects.

## Decision

Standardize on:

- Transport semantics: `at-least-once`.
- Application effect semantics: `exactly-once outcome via idempotent consumers`.

Idempotency rules:

- All external write requests must include `idempotency_key`.
- Consumers persist processed `event_id`/operation keys in dedup storage.
- Handlers must be replay-safe and deterministic.

Retry rules:

- Retries allowed for transient errors with bounded backoff.
- Non-retriable business failures produce terminal status events.

## Consequences

- Pros: robust failure recovery without requiring exactly-once transport guarantees.
- Cons: additional storage and logic for deduplication.
- Required follow-up: shared idempotency library and TTL/retention policy for dedup keys.

