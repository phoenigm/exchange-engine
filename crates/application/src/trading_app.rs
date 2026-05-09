use domain::trading::{DomainError, LimitOrder, MarketId, MatchResult, MatchingEngine};
use ports::MarketCatalog;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct PlaceOrderUseCase<C: MarketCatalog> {
    catalog: C,
    engines: Mutex<HashMap<MarketId, MatchingEngine>>,
}

impl<C: MarketCatalog> PlaceOrderUseCase<C> {
    pub fn new(catalog: C) -> Self {
        Self {
            catalog,
            engines: Mutex::new(HashMap::new()),
        }
    }

    pub fn execute(&self, market: &MarketId) -> Result<(), String> {
        if !self.catalog.is_enabled(market) {
            return Err("market is disabled".to_string());
        }
        Ok(())
    }

    pub fn place_limit_order(&self, order: LimitOrder) -> Result<MatchResult, DomainError> {
        if !self.catalog.is_enabled(&order.market) {
            return Err(DomainError::UnsupportedMarket(order.market.0.clone()));
        }
        let mut engines = self
            .engines
            .lock()
            .expect("matching engine map mutex poisoned");
        let market = order.market.clone();
        let engine = engines
            .entry(market.clone())
            .or_insert_with(|| MatchingEngine::new(market));
        engine.submit(order)
    }
}
