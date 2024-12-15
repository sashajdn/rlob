use thiserror::Error;

use super::book::Book;
use super::book::Order;
use super::book::Side;

pub struct LimitOrderBook {
    bids: Book,
    asks: Book,
}

pub struct OrderResult {
    order_id: String,
}

impl LimitOrderBook {
    pub fn new() -> Self {
        Self {
            bids: Book::new(Side::Buy),
            asks: Book::new(Side::Sell),
        }
    }

    pub fn place_order_in_bool(&mut self, order: Order) -> Result<OrderResult, PlaceOrderError> {
        match order {
            Order::Limit => match order.Side {
                Side::Buy => {}
                Side::Sell => {}
            },
            Order::Market => match order.Side {
                Side::Buy => {}
                Side::Sell => {}
            },
        }
    }
}

#[derive(Error, Debug)]
pub enum PlaceOrderError {
    #[error("time in force applied")]
    TimeInForceError,
    #[error("crosses the spread")]
    CrossedSpreadError,
}
