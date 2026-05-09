# ADR-05: Ledger Consistency Model

- Status: Accepted
- Priority: P0
- Date: 2026-05-09

## Context

Custodial exchange requires strict monetary correctness across deposits, withdrawals, settlements, fees, funding, and liquidation transfers.

## Decision

Use a strict `double-entry ledger` as monetary source of truth:

- Every balance mutation is represented as balanced debit/credit entries.
- No direct mutable balance updates outside ledger posting flows.
- Ledger posting is atomic within a DB transaction.

Account model:

- Separate account buckets for `available`, `locked`, and system accounts.
- Spot and perpetual ledgers are logically separated but reconciled at user/account level.

Reconciliation:

- Periodic checks:
  - ledger vs wallet on-chain holdings
  - ledger vs OMS/risk projections
- Mismatches generate incidents and automated containment workflows.

## Consequences

- Pros: auditable, replayable monetary history with strong invariants.
- Cons: more complex posting logic and operational reconciliation requirements.
- Required follow-up: document full posting matrix for each business event type.
