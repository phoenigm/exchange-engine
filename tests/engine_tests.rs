use exchange_engine::{
    ExecutionStatus, MatchingEngine, MarketId, OrderRequest, OrderType, Price, Quantity, Side,
};

fn req(
    market: &MarketId,
    user: u64,
    side: Side,
    order_type: OrderType,
    price: Option<u64>,
    qty: u64,
) -> OrderRequest {
    OrderRequest {
        market: market.clone(),
        user_id: user,
        side,
        order_type,
        price: price.map(Price::from_raw),
        qty: Quantity::from_raw(qty),
        time_in_force: None,
    }
}

#[test]
fn limit_buy_matches_resting_ask_price() {
    let mut engine = MatchingEngine::new();
    let market = engine.create_market("BTC", "USDT");

    let _ = engine.submit_order(req(&market, 1, Side::Ask, OrderType::Limit, Some(100), 2)).unwrap();
    let report = engine.submit_order(req(&market, 2, Side::Bid, OrderType::Limit, Some(105), 1)).unwrap();

    assert_eq!(report.status, ExecutionStatus::Filled);
    assert_eq!(report.filled_qty.as_u64(), 1);
    assert_eq!(report.trades[0].price.as_u64(), 100);
}

#[test]
fn partial_fill_puts_limit_remainder_to_book() {
    let mut engine = MatchingEngine::new();
    let market = engine.create_market("BTC", "USDT");

    let _ = engine.submit_order(req(&market, 1, Side::Ask, OrderType::Limit, Some(100), 1)).unwrap();
    let report = engine.submit_order(req(&market, 2, Side::Bid, OrderType::Limit, Some(100), 3)).unwrap();

    assert_eq!(report.status, ExecutionStatus::PartiallyFilled);
    assert_eq!(report.filled_qty.as_u64(), 1);
    assert_eq!(report.remaining_qty.as_u64(), 2);

    let snap = engine.order_book_snapshot(&market, 5).unwrap();
    assert_eq!(snap.bids.len(), 1);
    assert_eq!(snap.bids[0].price.as_u64(), 100);
    assert_eq!(snap.bids[0].qty.as_u64(), 2);
}

#[test]
fn market_order_consumes_multiple_levels_and_leaves_no_resting() {
    let mut engine = MatchingEngine::new();
    let market = engine.create_market("BTC", "USDT");

    let _ = engine.submit_order(req(&market, 1, Side::Ask, OrderType::Limit, Some(100), 1)).unwrap();
    let _ = engine.submit_order(req(&market, 2, Side::Ask, OrderType::Limit, Some(101), 1)).unwrap();
    let _ = engine.submit_order(req(&market, 3, Side::Ask, OrderType::Limit, Some(102), 10)).unwrap();

    let report = engine.submit_order(req(&market, 4, Side::Bid, OrderType::Market, None, 15)).unwrap();
    assert_eq!(report.status, ExecutionStatus::PartiallyFilled);
    assert_eq!(report.filled_qty.as_u64(), 12);
    assert_eq!(report.remaining_qty.as_u64(), 3);

    let snap = engine.order_book_snapshot(&market, 5).unwrap();
    assert!(snap.bids.is_empty());
    assert!(snap.asks.is_empty());
}

#[test]
fn price_time_priority_fifo_on_same_level() {
    let mut engine = MatchingEngine::new();
    let market = engine.create_market("BTC", "USDT");

    let _ = engine.submit_order(req(&market, 11, Side::Ask, OrderType::Limit, Some(100), 1)).unwrap();
    let _ = engine.submit_order(req(&market, 12, Side::Ask, OrderType::Limit, Some(100), 1)).unwrap();
    let report = engine.submit_order(req(&market, 13, Side::Bid, OrderType::Market, None, 2)).unwrap();

    assert_eq!(report.trades.len(), 2);
    assert_eq!(report.trades[0].maker_order_id, 1);
    assert_eq!(report.trades[1].maker_order_id, 2);
}

#[test]
fn markets_are_isolated() {
    let mut engine = MatchingEngine::new();
    let btc = engine.create_market("BTC", "USDT");
    let eth = engine.create_market("ETH", "USDT");

    let _ = engine.submit_order(req(&btc, 1, Side::Ask, OrderType::Limit, Some(100), 1)).unwrap();
    let _ = engine.submit_order(req(&btc, 2, Side::Bid, OrderType::Market, None, 1)).unwrap();
    let _ = engine.submit_order(req(&eth, 3, Side::Ask, OrderType::Limit, Some(200), 1)).unwrap();

    let btc_last = engine.last_price(&btc).unwrap().unwrap();
    let eth_last = engine.last_price(&eth).unwrap();

    assert_eq!(btc_last.as_u64(), 100);
    assert!(eth_last.is_none());
}

#[test]
fn slippage_estimate_and_vwap_match_expected_example() {
    let mut engine = MatchingEngine::new();
    let market = engine.create_market("BTC", "USDT");

    let _ = engine.submit_order(req(&market, 1, Side::Ask, OrderType::Limit, Some(100), 1)).unwrap();
    let _ = engine.submit_order(req(&market, 2, Side::Ask, OrderType::Limit, Some(101), 1)).unwrap();
    let _ = engine.submit_order(req(&market, 3, Side::Ask, OrderType::Limit, Some(102), 10)).unwrap();

    let est = engine.estimate_market_buy(&market, Quantity::from_raw(15)).unwrap();
    assert_eq!(est.fillable_qty.as_u64(), 12);
    assert_eq!(est.levels_consumed, 3);
    assert_eq!(est.total_cost_or_proceeds, 1221);
    assert_eq!(est.avg_price.unwrap().as_u64(), 101);

    let report = engine.submit_order(req(&market, 4, Side::Bid, OrderType::Market, None, 2)).unwrap();
    let vwap = MatchingEngine::vwap(&report.trades).unwrap().unwrap();
    assert_eq!(vwap.as_u64(), 100);
}

#[test]
fn top_of_book_and_mid_price_are_correct() {
    let mut engine = MatchingEngine::new();
    let market = engine.create_market("BTC", "USDT");

    let _ = engine.submit_order(req(&market, 1, Side::Bid, OrderType::Limit, Some(99), 2)).unwrap();
    let _ = engine.submit_order(req(&market, 2, Side::Bid, OrderType::Limit, Some(98), 2)).unwrap();
    let _ = engine.submit_order(req(&market, 3, Side::Ask, OrderType::Limit, Some(101), 2)).unwrap();
    let _ = engine.submit_order(req(&market, 4, Side::Ask, OrderType::Limit, Some(102), 2)).unwrap();

    let snap = engine.order_book_snapshot(&market, 5).unwrap();
    assert_eq!(snap.best_bid.unwrap().as_u64(), 99);
    assert_eq!(snap.best_ask.unwrap().as_u64(), 101);
    assert_eq!(snap.mid_price.unwrap().as_u64(), 100);
    assert_eq!(snap.bids.len(), 2);
    assert_eq!(snap.asks.len(), 2);
}
