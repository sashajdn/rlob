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
                let left_after_taking = qty - self.remaining_size;
                let taken = self.remaining_size;
                self.remaining_size = 0.0;
                (left_after_taking, self.remaining_size, taken)
            }
            Ordering::Less => {
                self.remaining_size -= qty;
                (0.0, self.remaining_size, qty)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_zero_qty() {
        let mut order = OrderContainer::new(0.0, 1);
        let (left_after_taking, order_remaining_size, taken) = order.take_qty(100.0);
        assert_eq!(left_after_taking, 100.0);
        assert_eq!(order_remaining_size, 0.0);
        assert_eq!(taken, 0.0);
    }

    #[test]
    fn test_take_qty_equal() {
        let mut order = OrderContainer::new(100.0, 1);
        let (left_after_taking, order_remaining_size, taken) = order.take_qty(100.0);
        assert_eq!(left_after_taking, 0.0);
        assert_eq!(order_remaining_size, 0.0);
        assert_eq!(taken, 100.0);
    }

    #[test]
    fn test_take_qty_greater() {
        let mut order = OrderContainer::new(100.0, 1);
        let (left_after_taking, order_remaining_size, taken) = order.take_qty(150.0);
        assert_eq!(left_after_taking, 50.0);
        assert_eq!(order_remaining_size, 0.0);
        assert_eq!(taken, 100.0);
    }

    #[test]
    fn test_take_qty_less() {
        let mut order = OrderContainer::new(100.0, 1);
        let (left_after_taking, order_remaining_size, taken) = order.take_qty(30.0);
        assert_eq!(left_after_taking, 0.0);
        assert_eq!(order_remaining_size, 70.0);
        assert_eq!(taken, 30.0);
    }
}
