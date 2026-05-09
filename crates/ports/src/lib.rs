use domain::trading::MarketId;

pub trait Clock: Send + Sync {
    fn now_unix_ms(&self) -> i64;
}

pub trait EventPublisher: Send + Sync {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), String>;
}

pub trait MarketCatalog: Send + Sync {
    fn is_enabled(&self, market: &MarketId) -> bool;
}

