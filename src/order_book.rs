use crate::errors::{EngineError, EngineResult};
use crate::types::{
    BidPrice, BookOrder, ExecutionEstimate, Level, OrderBookSnapshot, OrderId, Price, Quantity, Side,
    Timestamp, Trade,
};
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone)]
pub struct OrderBook {
    bids: BTreeMap<BidPrice, VecDeque<BookOrder>>,
    asks: BTreeMap<Price, VecDeque<BookOrder>>,
    pub last_price: Option<Price>,
}

impl Default for OrderBook {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_price: None,
        }
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.keys().next().map(|p| p.0)
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.keys().next().copied()
    }

    pub fn mid_price(&self) -> Option<Price> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(Price::from_raw((bid.as_u64() + ask.as_u64()) / 2)),
            _ => None,
        }
    }

    pub fn snapshot(&self, depth: usize) -> OrderBookSnapshot {
        let bids = self.top_n(Side::Bid, depth);
        let asks = self.top_n(Side::Ask, depth);
        OrderBookSnapshot {
            bids,
            asks,
            best_bid: self.best_bid(),
            best_ask: self.best_ask(),
            mid_price: self.mid_price(),
        }
    }

    pub fn top_n(&self, side: Side, depth: usize) -> Vec<Level> {
        if depth == 0 {
            return Vec::new();
        }

        let mut result = Vec::new();
        match side {
            Side::Bid => {
                for (price, orders) in self.bids.iter().take(depth) {
                    let total = orders.iter().map(|o| o.qty.as_u64()).sum::<u64>();
                    if total > 0 {
                        result.push(Level {
                            price: price.0,
                            qty: Quantity::from_raw(total),
                        });
                    }
                }
            }
            Side::Ask => {
                for (price, orders) in self.asks.iter().take(depth) {
                    let total = orders.iter().map(|o| o.qty.as_u64()).sum::<u64>();
                    if total > 0 {
                        result.push(Level {
                            price: *price,
                            qty: Quantity::from_raw(total),
                        });
                    }
                }
            }
        }
        result
    }

    pub fn add_resting_order(&mut self, order: BookOrder) {
        match order.side {
            Side::Bid => {
                self.bids
                    .entry(BidPrice(order.price))
                    .or_default()
                    .push_back(order);
            }
            Side::Ask => {
                self.asks.entry(order.price).or_default().push_back(order);
            }
        }
    }

    pub fn match_order(
        &mut self,
        taker_order_id: OrderId,
        taker_side: Side,
        mut qty: u64,
        limit_price: Option<Price>,
        ts: Timestamp,
    ) -> Vec<Trade> {
        let mut trades = Vec::new();

        while qty > 0 {
            let maybe_trade = match taker_side {
                Side::Bid => self.match_against_asks(taker_order_id, qty, limit_price, ts),
                Side::Ask => self.match_against_bids(taker_order_id, qty, limit_price, ts),
            };

            if let Some((trade, consumed)) = maybe_trade {
                qty -= consumed;
                self.last_price = Some(trade.price);
                trades.push(trade);
            } else {
                break;
            }
        }

        trades
    }

    fn match_against_asks(
        &mut self,
        taker_order_id: OrderId,
        remaining: u64,
        limit_price: Option<Price>,
        ts: Timestamp,
    ) -> Option<(Trade, u64)> {
        let best_price = *self.asks.keys().next()?;
        if let Some(limit) = limit_price {
            if best_price > limit {
                return None;
            }
        }

        let mut remove_level = false;
        let mut out = None;
        if let Some(level) = self.asks.get_mut(&best_price) {
            if let Some(maker) = level.front_mut() {
                let maker_qty = maker.qty.as_u64();
                let traded_qty = remaining.min(maker_qty);
                let new_maker_qty = maker_qty - traded_qty;
                maker.qty = Quantity::from_raw(new_maker_qty);

                let trade = Trade {
                    price: best_price,
                    qty: Quantity::from_raw(traded_qty),
                    maker_order_id: maker.id,
                    taker_order_id,
                    ts,
                };

                if new_maker_qty == 0 {
                    level.pop_front();
                }
                if level.is_empty() {
                    remove_level = true;
                }
                out = Some((trade, traded_qty));
            }
        }
        if remove_level {
            self.asks.remove(&best_price);
        }
        out
    }

    fn match_against_bids(
        &mut self,
        taker_order_id: OrderId,
        remaining: u64,
        limit_price: Option<Price>,
        ts: Timestamp,
    ) -> Option<(Trade, u64)> {
        let best_bid = *self.bids.keys().next()?;
        let best_price = best_bid.0;
        if let Some(limit) = limit_price {
            if best_price < limit {
                return None;
            }
        }

        let mut remove_level = false;
        let mut out = None;
        if let Some(level) = self.bids.get_mut(&best_bid) {
            if let Some(maker) = level.front_mut() {
                let maker_qty = maker.qty.as_u64();
                let traded_qty = remaining.min(maker_qty);
                let new_maker_qty = maker_qty - traded_qty;
                maker.qty = Quantity::from_raw(new_maker_qty);

                let trade = Trade {
                    price: best_price,
                    qty: Quantity::from_raw(traded_qty),
                    maker_order_id: maker.id,
                    taker_order_id,
                    ts,
                };

                if new_maker_qty == 0 {
                    level.pop_front();
                }
                if level.is_empty() {
                    remove_level = true;
                }
                out = Some((trade, traded_qty));
            }
        }
        if remove_level {
            self.bids.remove(&best_bid);
        }
        out
    }

    pub fn estimate_market(&self, side: Side, qty: Quantity) -> EngineResult<ExecutionEstimate> {
        let mut remaining = qty.as_u64();
        let mut fillable = 0u64;
        let mut total_notional: u128 = 0;
        let mut consumed_levels = 0usize;

        match side {
            Side::Bid => {
                for (price, orders) in &self.asks {
                    if remaining == 0 {
                        break;
                    }
                    let available = orders.iter().map(|o| o.qty.as_u64()).sum::<u64>();
                    let taken = available.min(remaining);
                    if taken == 0 {
                        continue;
                    }
                    consumed_levels += 1;
                    fillable += taken;
                    remaining -= taken;
                    total_notional = total_notional
                        .checked_add((price.as_u64() as u128) * (taken as u128))
                        .ok_or(EngineError::ArithmeticOverflow)?;
                }
            }
            Side::Ask => {
                for (price, orders) in &self.bids {
                    if remaining == 0 {
                        break;
                    }
                    let available = orders.iter().map(|o| o.qty.as_u64()).sum::<u64>();
                    let taken = available.min(remaining);
                    if taken == 0 {
                        continue;
                    }
                    consumed_levels += 1;
                    fillable += taken;
                    remaining -= taken;
                    total_notional = total_notional
                        .checked_add((price.0.as_u64() as u128) * (taken as u128))
                        .ok_or(EngineError::ArithmeticOverflow)?;
                }
            }
        }

        let avg_price = if fillable > 0 {
            Some(Price::from_raw((total_notional / fillable as u128) as u64))
        } else {
            None
        };

        Ok(ExecutionEstimate {
            requested_qty: qty,
            fillable_qty: Quantity::from_raw(fillable),
            avg_price,
            total_cost_or_proceeds: total_notional,
            levels_consumed: consumed_levels,
        })
    }
}
