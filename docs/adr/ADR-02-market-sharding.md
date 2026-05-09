# ADR-02: Market Sharding Strategy

- Status: Accepted
- Priority: P0
- Date: 2026-05-09

## Context

Initial markets are limited (`BTC/USDT`, `ETH/USDT`, `SOL/USDT`, `TON/USDT`), but architecture must support adding pairs and isolating hot markets as load grows.

## Decision

Use `market-based sharding` for matching:

- Baseline: each market is assigned to one logical shard with single-writer ownership.
- Initial deployment may place multiple logical shards on one node.
- Hot-market isolation: a market can be reassigned to a dedicated node without protocol changes.
- Routing key: `market_id` for command/event partitioning.

Rebalance approach:

- Pause new order intake for target market briefly.
- Drain in-flight commands.
- Snapshot and transfer market state.
- Resume on destination shard.

## Consequences

- Pros: deterministic ordering per market, easy horizontal scaling, clean hotspot handling.
- Cons: rebalancing requires operational tooling and strict runbooks.
- Required follow-up: define shard manager and migration SLOs.
