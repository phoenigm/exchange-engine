use domain::trading::MarketId;
use ports::MarketCatalog;

pub struct PlaceOrderUseCase<C: MarketCatalog> {
    catalog: C,
}

impl<C: MarketCatalog> PlaceOrderUseCase<C> {
    pub fn new(catalog: C) -> Self {
        Self { catalog }
    }

    pub fn execute(&self, market: &MarketId) -> Result<(), String> {
        if !self.catalog.is_enabled(market) {
            return Err("market is disabled".to_string());
        }
        Ok(())
    }
}
