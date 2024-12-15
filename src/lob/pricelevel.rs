use super::order::Order;

pub struct PriceLevel {
    pub price: f64,
    pub volume: f64,
    pub orders: Vec<Order>,
}

impl PriceLevel {
    pub fn new(price: f64, volume: f64) -> Self {
        Self {
            price,
            volume,
            orders: Vec::with_capacity(128),
        }
    }
}
