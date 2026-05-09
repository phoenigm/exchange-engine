use domain::trading::{LimitOrder, MarketId, MatchingEngine, Side};

fn buy(id: &str, price: u64, qty: u64) -> LimitOrder {
    LimitOrder {
        id: id.to_string(),
        market: MarketId("BTC/USDT".to_string()),
        user_id: "u1".to_string(),
        side: Side::Buy,
        price,
        qty,
    }
}

fn sell(id: &str, price: u64, qty: u64) -> LimitOrder {
    LimitOrder {
        id: id.to_string(),
        market: MarketId("BTC/USDT".to_string()),
        user_id: "u2".to_string(),
        side: Side::Sell,
        price,
        qty,
    }
}

#[test]
fn partially_fills_and_leaves_resting_order() {
    let mut engine = MatchingEngine::new(MarketId("BTC/USDT".to_string()));
    engine.submit(sell("s1", 62_000, 5)).expect("seed sell");

    let result = engine.submit(buy("b1", 62_500, 8)).expect("buy");

    assert_eq!(result.trades.len(), 1);
    assert_eq!(result.trades[0].qty, 5);
    assert_eq!(result.trades[0].price, 62_000);
    assert_eq!(result.remaining.expect("remaining order").qty, 3);
}

#[test]
fn honors_price_time_priority_on_same_level() {
    let mut engine = MatchingEngine::new(MarketId("BTC/USDT".to_string()));
    engine.submit(sell("s1", 62_000, 2)).expect("s1");
    engine.submit(sell("s2", 62_000, 3)).expect("s2");

    let result = engine.submit(buy("b1", 62_000, 4)).expect("buy");

    assert_eq!(result.trades.len(), 2);
    assert_eq!(result.trades[0].maker_order_id, "s1");
    assert_eq!(result.trades[0].qty, 2);
    assert_eq!(result.trades[1].maker_order_id, "s2");
    assert_eq!(result.trades[1].qty, 2);
}
