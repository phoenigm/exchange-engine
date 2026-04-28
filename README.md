# exchange-engine

In-memory CEX matching engine for multiple markets (for example `BTC/USDT`) with:
- limit and market orders
- price-time priority
- partial fills
- order book snapshots
- last trade price
- market slippage estimation

## Project Structure

- `src/lib.rs`: library entrypoint and exports.
- `src/engine.rs`: `MatchingEngine` and market registry.
- `src/order_book.rs`: order book internals and matching logic.
- `src/types.rs`: domain types and API structs.
- `src/errors.rs`: engine error model.
- `src/server.rs`: REST API server over the engine.
- `src/bin/server.rs`: thin executable entrypoint.
- `tests/engine_tests.rs`: integration tests for matching behavior.
- `scripts/run_server.sh`: start the server.
- `scripts/emulate_market.sh`: market behavior simulation through REST API.

## Quick Start

1. Start server (long-running process):

```bash
./scripts/run_server.sh
```

2. In another terminal run simulation via REST API:

```bash
./scripts/emulate_market.sh 300 42 40 http://127.0.0.1:8080
```

Arguments for `emulate_market.sh`:
- `300`: steps
- `42`: RNG seed
- `40`: delay in milliseconds between steps
- `http://127.0.0.1:8080`: base URL of REST API

## Development

Run tests:

```bash
cargo test
```

Build server binary:

```bash
cargo build --bin server
```

## API

Detailed REST API documentation: [docs/rest_api.md](C:\Users\Paradox\RustroverProjects\exchange-engine\docs\rest_api.md)
