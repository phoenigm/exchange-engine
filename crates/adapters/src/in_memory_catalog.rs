use std::collections::HashSet;

use domain::trading::MarketId;
use ports::MarketCatalog;

pub struct InMemoryMarketCatalog {
    enabled: HashSet<String>,
}

impl InMemoryMarketCatalog {
    pub fn new<I>(markets: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        Self {
            enabled: markets.into_iter().collect(),
        }
    }
}

impl MarketCatalog for InMemoryMarketCatalog {
    fn is_enabled(&self, market: &MarketId) -> bool {
        self.enabled.contains(&market.0)
    }
}

