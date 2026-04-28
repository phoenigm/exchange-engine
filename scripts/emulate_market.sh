#!/usr/bin/env bash
set -euo pipefail

STEPS="${1:-300}"
SEED="${2:-42}"
DELAY_MS="${3:-40}"
BASE_URL="${4:-http://127.0.0.1:8080}"

rand_state="$SEED"
next_user=1
reference_price=100000

rand_u32() {
  rand_state=$(( (1103515245 * rand_state + 12345) % 2147483648 ))
  echo "$rand_state"
}

rand_range() {
  local min="$1"
  local max="$2"
  local span=$((max - min + 1))
  local r
  r="$(rand_u32)"
  echo $(( min + (r % span) ))
}

extract_num() {
  local key="$1"
  local json="$2"
  echo "$json" | sed -n "s/.*\"$key\":\([0-9][0-9]*\).*/\1/p"
}

extract_nullable_num() {
  local key="$1"
  local json="$2"
  local value
  value="$(echo "$json" | sed -n "s/.*\"$key\":\([0-9][0-9]*\).*/\1/p")"
  if [[ -n "$value" ]]; then
    echo "$value"
  else
    echo "null"
  fi
}

extract_trades_count() {
  local json="$1"
  local content
  content="$(echo "$json" | sed -n 's/.*"trades":\[\(.*\)\].*/\1/p')"
  if [[ -z "$content" ]]; then
    echo 0
    return
  fi
  local count
  count="$(echo "$content" | grep -o "maker_order_id" | wc -l | tr -d ' ')"
  echo "${count:-0}"
}

wait_api() {
  local attempts=30
  for ((i=1; i<=attempts; i++)); do
    if curl -sS --fail "$BASE_URL/health" >/dev/null 2>&1; then
      return
    fi
    sleep 0.5
  done
  echo "API is not reachable at $BASE_URL after waiting" >&2
  exit 1
}

post_json() {
  local path="$1"
  local payload="$2"
  curl -sS --fail -X POST "$BASE_URL$path" \
    -H "Content-Type: application/json" \
    -d "$payload"
}

get_json() {
  local path="$1"
  curl -sS --fail "$BASE_URL$path"
}

submit_order() {
  local side="$1"
  local order_type="$2"
  local price="$3"
  local qty="$4"
  local user_id="$5"

  local payload
  if [[ "$price" == "null" ]]; then
    payload="{\"market\":{\"base\":\"BTC\",\"quote\":\"USDT\"},\"user_id\":$user_id,\"side\":\"$side\",\"order_type\":\"$order_type\",\"price\":null,\"qty\":$qty}"
  else
    payload="{\"market\":{\"base\":\"BTC\",\"quote\":\"USDT\"},\"user_id\":$user_id,\"side\":\"$side\",\"order_type\":\"$order_type\",\"price\":$price,\"qty\":$qty}"
  fi
  post_json "/orders" "$payload"
}

wait_api
post_json "/markets" '{"base":"BTC","quote":"USDT"}' >/dev/null

for i in $(seq 1 12); do
  qty=$(( (i % 5) + 2 ))
  bid_price=$((100000 - i * 20))
  ask_price=$((100000 + i * 20))
  submit_order "bid" "limit" "$bid_price" "$qty" "$next_user" >/dev/null
  next_user=$((next_user + 1))
  submit_order "ask" "limit" "$ask_price" "$qty" "$next_user" >/dev/null
  next_user=$((next_user + 1))
done

echo "API SIM START steps=$STEPS seed=$SEED market=BTC/USDT"
for step in $(seq 1 "$STEPS"); do
  drift="$(rand_range -50 50)"
  reference_price=$((reference_price + drift))
  if (( reference_price < 90000 )); then
    reference_price=90000
  fi

  action_roll="$(rand_range 0 99)"
  side_pick="$(rand_range 0 1)"
  if (( side_pick == 0 )); then
    side="bid"
  else
    side="ask"
  fi

  if (( action_roll < 30 )); then
    order_type="market"
    qty="$(rand_range 1 5)"
    price="null"
  else
    order_type="limit"
    qty="$(rand_range 1 8)"
    edge="$(rand_range 5 120)"
    if [[ "$side" == "bid" ]]; then
      price=$((reference_price - edge))
    else
      price=$((reference_price + edge))
    fi
  fi

  report="$(submit_order "$side" "$order_type" "$price" "$qty" "$next_user")"
  next_user=$((next_user + 1))
  snapshot="$(get_json "/markets/BTC/USDT/snapshot?depth=5")"

  filled="$(extract_num "filled_qty" "$report")"
  remaining="$(extract_num "remaining_qty" "$report")"
  trades_count="$(extract_trades_count "$report")"
  last_price="$(extract_nullable_num "last_price_after" "$report")"
  best_bid="$(extract_nullable_num "best_bid" "$snapshot")"
  best_ask="$(extract_nullable_num "best_ask" "$snapshot")"

  spread="null"
  if [[ "$best_bid" != "null" && "$best_ask" != "null" ]]; then
    spread=$((best_ask - best_bid))
  fi

  printf "step=%04d side=%s type=%s filled=%s rem=%s trades=%s last=%s bid=%s ask=%s spread=%s\n" \
    "$step" "$side" "$order_type" "${filled:-0}" "${remaining:-0}" "$trades_count" "$last_price" "$best_bid" "$best_ask" "$spread"

  sleep "$(awk "BEGIN {printf \"%.3f\", $DELAY_MS/1000}")"
done
echo "API SIM DONE"
