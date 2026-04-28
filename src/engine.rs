use crate::errors::{EngineError, EngineResult};
use crate::order_book::OrderBook;
use crate::types::{
    BookOrder, ExecutionReport, ExecutionStatus, MarketId, OrderRequest, OrderType, OrderId, Price,
    Quantity, Side, TimeInForce, Timestamp, Trade,
};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default)]
pub struct MatchingEngine {
    markets: HashMap<MarketId, OrderBook>,
    next_order_id: OrderId,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            markets: HashMap::new(),
            next_order_id: 1,
        }
    }

    pub fn create_market(&mut self, base: impl Into<String>, quote: impl Into<String>) -> MarketId {
        let market = MarketId::new(base, quote);
        self.markets.entry(market.clone()).or_default();
        market
    }

    pub fn submit_order(&mut self, request: OrderRequest) -> EngineResult<ExecutionReport> {
        self.validate_order(&request)?;
        let order_id = self.allocate_order_id();
        let ts = now_millis();
        let book = self
            .markets
            .get_mut(&request.market)
            .ok_or(EngineError::UnknownMarket)?;

        let limit_price = if request.order_type == OrderType::Limit {
            request.price
        } else {
            None
        };

        let requested_qty = request.qty.as_u64();
        let trades = book.match_order(order_id, request.side, requested_qty, limit_price, ts);
        let filled_qty_raw: u64 = trades.iter().map(|t| t.qty.as_u64()).sum();
        let remaining_qty_raw = requested_qty - filled_qty_raw;

        if request.order_type == OrderType::Limit && remaining_qty_raw > 0 {
            let resting = BookOrder {
                id: order_id,
                user_id: request.user_id,
                side: request.side,
                price: request.price.expect("limit price validated"),
                qty: Quantity::from_raw(remaining_qty_raw),
                ts,
            };
            book.add_resting_order(resting);
        }

        let status = if filled_qty_raw == requested_qty {
            ExecutionStatus::Filled
        } else if filled_qty_raw > 0 {
            ExecutionStatus::PartiallyFilled
        } else if request.order_type == OrderType::Limit {
            ExecutionStatus::Open
        } else {
            ExecutionStatus::Rejected
        };

        Ok(ExecutionReport {
            accepted: true,
            trades,
            filled_qty: Quantity::from_raw(filled_qty_raw),
            remaining_qty: Quantity::from_raw(remaining_qty_raw),
            status,
            last_price_after: book.last_price,
        })
    }

    pub fn order_book_snapshot(
        &self,
        market: &MarketId,
        depth: usize,
    ) -> EngineResult<crate::types::OrderBookSnapshot> {
        let book = self.markets.get(market).ok_or(EngineError::UnknownMarket)?;
        Ok(book.snapshot(depth))
    }

    pub fn last_price(&self, market: &MarketId) -> EngineResult<Option<Price>> {
        let book = self.markets.get(market).ok_or(EngineError::UnknownMarket)?;
        Ok(book.last_price)
    }

    pub fn estimate_market_buy(
        &self,
        market: &MarketId,
        qty: Quantity,
    ) -> EngineResult<crate::types::ExecutionEstimate> {
        let book = self.markets.get(market).ok_or(EngineError::UnknownMarket)?;
        book.estimate_market(Side::Bid, qty)
    }

    pub fn estimate_market_sell(
        &self,
        market: &MarketId,
        qty: Quantity,
    ) -> EngineResult<crate::types::ExecutionEstimate> {
        let book = self.markets.get(market).ok_or(EngineError::UnknownMarket)?;
        book.estimate_market(Side::Ask, qty)
    }

    pub fn vwap(trades: &[Trade]) -> EngineResult<Option<Price>> {
        if trades.is_empty() {
            return Ok(None);
        }
        let mut notional: u128 = 0;
        let mut qty_total: u128 = 0;
        for t in trades {
            notional = notional
                .checked_add((t.price.as_u64() as u128) * (t.qty.as_u64() as u128))
                .ok_or(EngineError::ArithmeticOverflow)?;
            qty_total = qty_total
                .checked_add(t.qty.as_u64() as u128)
                .ok_or(EngineError::ArithmeticOverflow)?;
        }
        if qty_total == 0 {
            return Ok(None);
        }
        Ok(Some(Price::from_raw((notional / qty_total) as u64)))
    }

    fn validate_order(&self, request: &OrderRequest) -> EngineResult<()> {
        if request.qty.as_u64() == 0 {
            return Err(EngineError::InvalidQuantity);
        }

        match request.order_type {
            OrderType::Limit => {
                if request.price.is_none() {
                    return Err(EngineError::InvalidPriceForLimit);
                }
            }
            OrderType::Market => {
                if request.price.is_some() {
                    return Err(EngineError::PriceNotAllowedForMarket);
                }
                if request.time_in_force.is_some_and(|tif| tif != TimeInForce::Ioc) {
                    return Err(EngineError::PriceNotAllowedForMarket);
                }
            }
        }
        Ok(())
    }

    fn allocate_order_id(&mut self) -> OrderId {
        let id = self.next_order_id;
        self.next_order_id = self.next_order_id.saturating_add(1);
        id
    }
}

fn now_millis() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
