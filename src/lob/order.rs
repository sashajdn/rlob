use super::book::Side;

pub struct LimitOrderParams {
    limit_price: f64,
    side: Side,
    quantity: f64,
}

impl LimitOrderParams {
    pub fn new(limit_price: f64, side: Side, quantity: f64) -> Self {
        Self {
            limit_price,
            side,
            quantity,
        }
    }
}

pub struct MarketOrderParams {
    side: Side,
    quantity: f64,
}

impl MarketOrderParams {
    pub fn new(side: Side, quantity: f64) -> Self {
        Self { side, quantity }
    }
}

pub enum Order {
    Limit(LimitOrderParams),
    Market(MarketOrderParams),
}
