use crate::errors::{EngineError, EngineResult};
use std::cmp::Ordering;

pub type OrderId = u64;
pub type UserId = u64;
pub type Timestamp = u64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarketId {
    pub base: String,
    pub quote: String,
}

impl MarketId {
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            quote: quote.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Price(u64);

impl Price {
    pub fn new(value: u64) -> EngineResult<Self> {
        if value == 0 {
            return Err(EngineError::InvalidPriceForLimit);
        }
        Ok(Self(value))
    }

    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity(u64);

impl Quantity {
    pub fn new(value: u64) -> EngineResult<Self> {
        if value == 0 {
            return Err(EngineError::InvalidQuantity);
        }
        Ok(Self(value))
    }

    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    Gtc,
    Ioc,
}

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub market: MarketId,
    pub user_id: UserId,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Option<Price>,
    pub qty: Quantity,
    pub time_in_force: Option<TimeInForce>,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub price: Price,
    pub qty: Quantity,
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
    pub ts: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Filled,
    PartiallyFilled,
    Open,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct ExecutionReport {
    pub accepted: bool,
    pub trades: Vec<Trade>,
    pub filled_qty: Quantity,
    pub remaining_qty: Quantity,
    pub status: ExecutionStatus,
    pub last_price_after: Option<Price>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level {
    pub price: Price,
    pub qty: Quantity,
}

#[derive(Debug, Clone)]
pub struct OrderBookSnapshot {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    pub best_bid: Option<Price>,
    pub best_ask: Option<Price>,
    pub mid_price: Option<Price>,
}

#[derive(Debug, Clone)]
pub struct ExecutionEstimate {
    pub requested_qty: Quantity,
    pub fillable_qty: Quantity,
    pub avg_price: Option<Price>,
    pub total_cost_or_proceeds: u128,
    pub levels_consumed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookOrder {
    pub id: OrderId,
    pub user_id: UserId,
    pub side: Side,
    pub price: Price,
    pub qty: Quantity,
    pub ts: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BidPrice(pub Price);

impl Ord for BidPrice {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl PartialOrd for BidPrice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
