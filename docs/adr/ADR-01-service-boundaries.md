# ADR-01: Service Boundaries and Ownership

- Status: Accepted
- Priority: P0
- Date: 2026-05-09

## Context

The platform includes spot and perpetual trading, custodial wallets, and strict balance correctness requirements.  
Clear service ownership is required to avoid duplicated logic and inconsistent invariants.

## Decision

Adopt the following service boundaries:

- `api-gateway`: edge routing, auth enforcement, rate limiting.
- `identity-auth`: user identity, API keys, scopes, session validation.
- `trading-api`: external trading endpoints, request validation, orchestration.
- `risk-margin`: pre-trade checks, margin checks, risk snapshots.
- `matching-engine`: market matching logic, deterministic execution events.
- `oms`: order lifecycle projection and query model.
- `ledger`: double-entry postings, monetary source of truth.
- `wallet`: deposit detection, withdrawal workflow, blockchain integration.
- `market-data`: order book/trades/ticker streaming.
- `liquidation-adl`: liquidation triggers and execution pipeline.
- `reference-data`: pair metadata, limits, market configs.

Ownership rules:

- Monetary invariants are owned by `ledger`.
- Matching state and match priority are owned by `matching-engine`.
- Risk formulas and margin policies are owned by `risk-margin`.
- API contracts and idempotency keys at ingress are owned by `trading-api`.

## Consequences

- Pros: clear ownership, independent scaling, reduced cross-service ambiguity.
- Cons: more service coordination and event-driven integration complexity.
- Required follow-up: define shared domain contracts crate and event schemas.

