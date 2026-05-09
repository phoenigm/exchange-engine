use std::cmp::min;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarketId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitOrder {
    pub id: String,
    pub market: MarketId,
    pub user_id: String,
    pub side: Side,
    pub price: u64,
    pub qty: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trade {
    pub market: MarketId,
    pub taker_order_id: String,
    pub maker_order_id: String,
    pub price: u64,
    pub qty: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    InvalidOrder(&'static str),
    UnsupportedMarket(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub trades: Vec<Trade>,
    pub remaining: Option<LimitOrder>,
}

#[derive(Debug)]
pub struct MatchingEngine {
    market: MarketId,
    bids: BTreeMap<u64, VecDeque<LimitOrder>>,
    asks: BTreeMap<u64, VecDeque<LimitOrder>>,
}

impl MatchingEngine {
    pub fn new(market: MarketId) -> Self {
        Self {
            market,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn submit(&mut self, order: LimitOrder) -> Result<MatchResult, DomainError> {
        self.validate_order(&order)?;

        let mut taker = order;
        let mut trades = Vec::new();

        match taker.side {
            Side::Buy => self.match_buy(&mut taker, &mut trades),
            Side::Sell => self.match_sell(&mut taker, &mut trades),
        }

        let remaining = if taker.qty > 0 {
            match taker.side {
                Side::Buy => self.bids.entry(taker.price).or_default().push_back(taker.clone()),
                Side::Sell => self.asks.entry(taker.price).or_default().push_back(taker.clone()),
            }
            Some(taker)
        } else {
            None
        };

        Ok(MatchResult { trades, remaining })
    }

    fn validate_order(&self, order: &LimitOrder) -> Result<(), DomainError> {
        if order.market != self.market {
            return Err(DomainError::UnsupportedMarket(order.market.0.clone()));
        }
        if order.id.is_empty() || order.user_id.is_empty() {
            return Err(DomainError::InvalidOrder("id and user_id are required"));
        }
        if order.price == 0 || order.qty == 0 {
            return Err(DomainError::InvalidOrder("price and qty must be > 0"));
        }
        Ok(())
    }

    fn match_buy(&mut self, taker: &mut LimitOrder, trades: &mut Vec<Trade>) {
        while taker.qty > 0 {
            let Some((&best_ask, _)) = self.asks.iter().next() else {
                break;
            };
            if best_ask > taker.price {
                break;
            }

            let queue = self.asks.get_mut(&best_ask).expect("ask level must exist");
            while taker.qty > 0 {
                let Some(maker) = queue.front_mut() else {
                    break;
                };

                let traded = min(taker.qty, maker.qty);
                maker.qty -= traded;
                taker.qty -= traded;

                trades.push(Trade {
                    market: taker.market.clone(),
                    taker_order_id: taker.id.clone(),
                    maker_order_id: maker.id.clone(),
                    price: best_ask,
                    qty: traded,
                });

                if maker.qty == 0 {
                    queue.pop_front();
                }
            }

            if queue.is_empty() {
                self.asks.remove(&best_ask);
            }
        }
    }

    fn match_sell(&mut self, taker: &mut LimitOrder, trades: &mut Vec<Trade>) {
        while taker.qty > 0 {
            let Some((&best_bid, _)) = self.bids.iter().next_back() else {
                break;
            };
            if best_bid < taker.price {
                break;
            }

            let queue = self.bids.get_mut(&best_bid).expect("bid level must exist");
            while taker.qty > 0 {
                let Some(maker) = queue.front_mut() else {
                    break;
                };

                let traded = min(taker.qty, maker.qty);
                maker.qty -= traded;
                taker.qty -= traded;

                trades.push(Trade {
                    market: taker.market.clone(),
                    taker_order_id: taker.id.clone(),
                    maker_order_id: maker.id.clone(),
                    price: best_bid,
                    qty: traded,
                });

                if maker.qty == 0 {
                    queue.pop_front();
                }
            }

            if queue.is_empty() {
                self.bids.remove(&best_bid);
            }
        }
    }
}
