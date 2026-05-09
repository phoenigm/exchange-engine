# ADR-07: Database Topology and High Availability

- Status: Accepted
- Priority: P0
- Date: 2026-05-09

## Context

Target SLA is `99.9%` in a single region with multi-AZ deployment.  
Relational storage is required for ledger, orders projection, wallets, and reference data.

## Decision

Adopt PostgreSQL HA topology:

- Single writable primary in AZ-A.
- At least two synchronous/near-synchronous replicas across AZ-B/AZ-C.
- Read traffic routed to replicas where consistency allows.

Failover policy:

- Automated failover with fencing and split-brain prevention.
- Application-level retry and connection pool reset on failover event.

Backup/recovery:

- Continuous WAL archiving.
- Daily full backups + frequent incremental snapshots.
- Regular restore drills.

Targets:

- RPO: <= 60 seconds
- RTO: <= 15 minutes

## Consequences

- Pros: practical HA baseline for MVP with clear DR posture.
- Cons: write scaling remains vertically constrained at primary.
- Required follow-up: partitioning strategy for highest-growth tables and retention policy.

