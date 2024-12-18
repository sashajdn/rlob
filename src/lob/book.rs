use crate::lob::order::OrderContainer;
use crate::lob::pricelevel::{FillEvents, PriceLevel, PriceLevels, TakeError};

const MAX_DEPTH: usize = 2 << 16; // 65536

pub struct Book {
    side: Side,
    price_levels: PriceLevels,
}

impl Book {
    pub fn new(side: Side) -> Self {
        Self {
            side,
            price_levels: PriceLevels::new(MAX_DEPTH),
        }
    }

    pub fn place_maker_limit_order(&mut self, price: f64, order: OrderContainer) {
        self.price_levels.make(price, order);
    }

    pub fn place_taker_market_order(
        &mut self,
        order: OrderContainer,
    ) -> Result<FillEvents, TakeError> {
        self.price_levels.take(order.size)
    }

    pub fn depth(&self) -> f64 {
        0.0
    }
}

#[derive(Debug, Clone)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn price_levels_comparator(&self) -> fn(&PriceLevel, &PriceLevel) -> std::cmp::Ordering {
        match self {
            Side::Buy => |a, b| a.price.total_cmp(&b.price),
            Side::Sell => |a, b| b.price.total_cmp(&a.price),
        }
    }
}
