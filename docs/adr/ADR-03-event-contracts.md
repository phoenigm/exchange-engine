# ADR-03: Event Contract Format and Schema Evolution

- Status: Accepted
- Priority: P0
- Date: 2026-05-09

## Context

Core services communicate through events. Contracts must support reliable evolution without breaking consumers.

## Decision

Adopt two-phase event format strategy:

- MVP: JSON payloads with explicit envelope metadata.
- Scale phase: migrate to `protobuf` with schema registry.

Mandatory envelope fields:

- `event_id` (globally unique)
- `event_type`
- `event_version`
- `occurred_at`
- `producer_service`
- `idempotency_key` (for externally triggered flows)

Schema policy:

- Backward compatible additive changes only within major version.
- Breaking changes require new `event_version` and dual-read migration period.

## Consequences

- Pros: fast MVP delivery while preserving migration path to stronger schema governance.
- Cons: JSON requires tighter runtime validation discipline early on.
- Required follow-up: publish event catalog and compatibility checklist in CI.

