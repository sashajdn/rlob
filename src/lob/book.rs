use super::order::Order;

pub struct Book {
    side: Side,
}

impl Book {
    pub fn new(side: Side) -> Self {
        Self { side }
    }

    pub fn place_limit_order(&mut self, order: Order::Limit) {}

    pub fn place_market_order(&mut self, order: Order::Market) {}
}

#[derive(Debug, Clone)]
pub enum Side {
    Buy,
    Sell,
}
