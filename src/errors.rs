use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    UnknownMarket,
    InvalidQuantity,
    InvalidPriceForLimit,
    PriceNotAllowedForMarket,
    ArithmeticOverflow,
}

pub type EngineResult<T> = Result<T, EngineError>;

impl Display for EngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::UnknownMarket => write!(f, "unknown market"),
            EngineError::InvalidQuantity => write!(f, "invalid quantity"),
            EngineError::InvalidPriceForLimit => write!(f, "invalid price for limit order"),
            EngineError::PriceNotAllowedForMarket => {
                write!(f, "price must not be set for market order")
            }
            EngineError::ArithmeticOverflow => write!(f, "arithmetic overflow"),
        }
    }
}

impl std::error::Error for EngineError {}
