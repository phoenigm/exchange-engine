# ADR-06: Perpetual Risk and Margin Model (MVP)

- Status: Accepted
- Priority: P0
- Date: 2026-05-09

## Context

Perpetual futures require consistent margin checks, liquidation triggers, and funding logic under low-latency constraints.

## Decision

For MVP:

- Margin model: isolated margin per position.
- Risk checks:
  - Initial Margin (IM) at order acceptance.
  - Maintenance Margin (MM) for liquidation trigger.
- Mark price:
  - external index price + bounded smoothing mechanism.
- Funding:
  - periodic funding intervals with ledger postings between longs/shorts.

Liquidation baseline:

- Trigger when equity falls below MM requirement.
- Use staged liquidation (partial first, then full close if needed).
- Bankruptcy handling routes losses to insurance account (ADL expansion later).

## Consequences

- Pros: simpler and safer MVP risk model with predictable behavior.
- Cons: isolated margin may be less capital-efficient than cross margin.
- Required follow-up: define exact IM/MM formulas, leverage tiers, and mark-price guardrails per market.
