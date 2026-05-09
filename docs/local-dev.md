# Local Dev Runbook

Use `Makefile` commands:

```bash
make up
make down
make status
make test
make build
```

Requirements:
- `make`
- `sh`/`bash` environment (`nohup`, `kill`)

`make up`:
- builds workspace;
- starts `trading-api`, `market-data-api`, `wallet-worker`, `risk-worker`, `liquidation-worker`;
- stores logs in `.local/dev-runtime/logs`;
- stores process IDs in `.local/dev-runtime/pids/*.pid`.
