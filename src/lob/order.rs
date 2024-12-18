use super::book::Side;
use crate::lob::sequencer::OrderID;
use std::cmp::Ordering;

pub struct LimitOrderParams {
    pub limit_price: f64,
    pub side: Side,
    pub quantity: f64,
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
    pub side: Side,
    pub quantity: f64,
}

impl MarketOrderParams {
    pub fn new(side: Side, quantity: f64) -> Self {
        Self { side, quantity }
    }
}

pub enum OrderRequest {
    Limit(LimitOrderParams),
    Market(MarketOrderParams),
}

#[derive(Debug)]
pub struct OrderContainer {
    pub size: f64,
    pub order_id: OrderID,
    remaining_size: f64,
}

impl OrderContainer {
    pub fn new(size: f64, order_id: OrderID) -> Self {
        Self {
            size,
            order_id,
            remaining_size: size,
        }
    }

    pub fn take_qty(&mut self, qty: f64) -> (f64, f64, f64) {
        if self.remaining_size == 0.0 {
            return (qty, 0.0, 0.0);
        }

        match qty.total_cmp(&self.remaining_size) {
            Ordering::Equal => {
                self.remaining_size = 0.0;
                (0.0, 0.0, qty)
            }
            Ordering::Greater => {
                let left = qty - self.remaining_size;
                let taken = self.remaining_size;
                self.remaining_size = 0.0;
                (left, self.remaining_size, taken)
            }
            Ordering::Less => {
                let taken = self.remaining_size - qty;
                self.remaining_size -= qty;
                (0.0, self.remaining_size - qty, taken)
            }
        }
    }
}
