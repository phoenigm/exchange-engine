# REST API

Base URL: `http://127.0.0.1:8080`

## Health

### `GET /health`

Response:

```json
{ "status": "ok" }
```

## Markets

### `POST /markets`

Create market if it does not exist.

Request:

```json
{
  "base": "BTC",
  "quote": "USDT"
}
```

Response:

```json
{
  "market": {
    "base": "BTC",
    "quote": "USDT"
  }
}
```

## Orders

### `POST /orders`

Create and process order.

Request:

```json
{
  "market": { "base": "BTC", "quote": "USDT" },
  "user_id": 101,
  "side": "bid",
  "order_type": "limit",
  "price": 100000,
  "qty": 2
}
```

Fields:
- `side`: `bid|buy|ask|sell`
- `order_type`: `limit|market`
- `price`: required for `limit`, must be `null` for `market`

Response:

```json
{
  "accepted": true,
  "filled_qty": 1,
  "remaining_qty": 1,
  "status": "PartiallyFilled",
  "last_price_after": 100000,
  "trades": [
    {
      "price": 100000,
      "qty": 1,
      "maker_order_id": 1,
      "taker_order_id": 2,
      "ts": 1714320000000
    }
  ]
}
```

## Market Data

### `GET /markets/{base}/{quote}/snapshot?depth=5`

Response:

```json
{
  "best_bid": 99980,
  "best_ask": 100020,
  "mid_price": 100000,
  "bids": [{ "price": 99980, "qty": 5 }],
  "asks": [{ "price": 100020, "qty": 4 }]
}
```

### `GET /markets/{base}/{quote}/last-price`

Response:

```json
{ "last_price": 100020 }
```

## Errors

Format:

```json
{ "error": "message" }
```

Status mapping:
- `400`: validation errors
- `404`: unknown market / route
- `500`: internal errors

## cURL Examples

Create market:

```bash
curl -X POST http://127.0.0.1:8080/markets \
  -H "Content-Type: application/json" \
  -d '{"base":"BTC","quote":"USDT"}'
```

Place limit order:

```bash
curl -X POST http://127.0.0.1:8080/orders \
  -H "Content-Type: application/json" \
  -d '{"market":{"base":"BTC","quote":"USDT"},"user_id":1,"side":"bid","order_type":"limit","price":100000,"qty":3}'
```

Snapshot:

```bash
curl "http://127.0.0.1:8080/markets/BTC/USDT/snapshot?depth=5"
```
