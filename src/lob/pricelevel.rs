use super::book::Side;
use super::order::OrderContainer;
use crate::lob::sequencer::OrderID;

use thiserror::Error;

pub struct PriceLevels {
    levels: Vec<PriceLevel>,
    side: Side,
}

impl PriceLevels {
    pub fn new(depth: usize) -> Self {
        Self {
            levels: Vec::with_capacity(depth),
            side: Side::Buy,
        }
    }

    pub fn make(&mut self, price: f64, order: OrderContainer) {
        if let Some(price_level) = self.levels.iter_mut().find(|level| level.price == price) {
            price_level.make(order);
        } else {
            let mut price_level = PriceLevel::new(price, 0.0);
            price_level.make(order);
            self.levels.push(price_level);
        }

        self.levels.sort_by(self.side.price_levels_comparator());
    }

    pub fn take(&mut self, qty: f64) -> Result<FillEvents, TakeError> {
        if qty == 0.0 {
            return Err(TakeError::ZeroQuantity);
        }

        if self.levels.is_empty() {
            return Err(TakeError::EmptyBook);
        }

        let mut total_fill_events = FillEvents::new(32);
        let mut remaining_qty = qty;

        for pl in &mut self.levels {
            let (fill_events, remaining_qty_after_take) = pl.take(remaining_qty);
            total_fill_events.0.extend(fill_events.0);
            remaining_qty = remaining_qty_after_take;

            if remaining_qty == 0.0 {
                return Ok(total_fill_events);
            }
        }

        Ok(total_fill_events)
    }
}

#[derive(Debug)]
pub struct PriceLevel {
    pub price: f64,
    pub volume: f64,
    pub orders: Vec<OrderContainer>,
}

impl PriceLevel {
    #[inline]
    pub fn new(price: f64, volume: f64) -> Self {
        Self {
            price,
            volume,
            orders: Vec::with_capacity(128),
        }
    }

    #[inline]
    pub fn make(&mut self, order: OrderContainer) {
        self.volume += order.size;
        self.orders.push(order);
    }

    #[inline]
    pub fn take(&mut self, qty: f64) -> (FillEvents, f64) {
        let mut order_idx_to_drain_upto: usize = 0;
        let mut total_taken = 0.0;
        let mut remaining_qty = qty;

        let mut fill_events = FillEvents::new(self.orders.len());

        for order in &mut self.orders {
            let (qty_left_after_taking, order_remaining_size, taken) =
                order.take_qty(remaining_qty);
            remaining_qty = qty_left_after_taking;

            if taken == 0.0 {
                break;
            }

            total_taken += taken;
            self.volume -= taken;

            let order_id = std::mem::take(&mut order.order_id);
            fill_events.0.push(FillEvent {
                order_id,
                size: taken,
            });

            if order_remaining_size <= 0.0 {
                order_idx_to_drain_upto += 1;
            }

            if remaining_qty <= 0.0 {
                break;
            }
        }

        self.orders.drain(0..order_idx_to_drain_upto);
        (fill_events, qty - total_taken)
    }

    pub fn num_orders_in_queue(&self) -> usize {
        self.orders.len()
    }
}

pub struct FillEvents(Vec<FillEvent>);

impl FillEvents {
    pub fn new(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

pub struct FillEvent {
    order_id: OrderID,
    size: f64,
}

#[derive(Error, Debug)]
pub enum TakeError {
    #[error("empty book")]
    EmptyBook,
    #[error("not enough volume in book")]
    NotEnoughVolume,
    #[error("cannot take zero quantity")]
    ZeroQuantity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_new_price_level() {
        let mut pl = PriceLevel::new(10.0, 0.0);
        let order1 = OrderContainer::new(100.0, 1);
        let order2 = OrderContainer::new(201.0, 2);

        pl.make(order1);
        assert_eq!(pl.volume, 100.0);
        assert_eq!(pl.price, 10.0);
        assert_eq!(pl.num_orders_in_queue(), 1);

        pl.make(order2);
        assert_eq!(pl.volume, 301.0);
        assert_eq!(pl.price, 10.0);
        assert_eq!(pl.num_orders_in_queue(), 2);
    }

    #[test]
    fn test_multi_take_from_existing_price_level() {
        let mut pl = PriceLevel::new(10.0, 0.0);
        let order1 = OrderContainer::new(100.0, 1);
        let order2 = OrderContainer::new(200.0, 2);

        pl.make(order1);
        pl.make(order2);
        assert_eq!(pl.volume, 300.0);

        let (fill_events, left_after_taking) = pl.take(50.0);
        assert_eq!(fill_events.0.len(), 1);
        assert_eq!(left_after_taking, 0.0);
        assert_eq!(pl.num_orders_in_queue(), 2);
        assert_eq!(pl.volume, 250.0);

        pl.take(50.0);
        assert_eq!(fill_events.0.len(), 1);
        assert_eq!(left_after_taking, 0.0);
        assert_eq!(pl.num_orders_in_queue(), 1);
        assert_eq!(pl.volume, 200.0);

        let (fill_events, left_after_taking) = pl.take(250.0);
        assert_eq!(fill_events.0.len(), 1);
        assert_eq!(left_after_taking, 50.0);
        assert_eq!(pl.num_orders_in_queue(), 0);
        assert_eq!(pl.volume, 0.0);
    }

    #[test]
    fn test_take_price_level_with_too_small_liquidity() {
        let mut pl = PriceLevel::new(10.0, 0.0);
        let order = OrderContainer::new(10.0, 1);
        pl.make(order);
        assert_eq!(pl.volume, 10.0);
        assert_eq!(pl.num_orders_in_queue(), 1);

        let (fill_events, left_after_taking) = pl.take(11.0);
        assert_eq!(pl.volume, 0.0);
        assert_eq!(left_after_taking, 1.0);
        assert_eq!(fill_events.0.len(), 1);
        assert_eq!(pl.num_orders_in_queue(), 0);
    }
}
