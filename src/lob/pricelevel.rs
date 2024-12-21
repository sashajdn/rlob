use crate::lob::book::{FillEvent, FillEvents};
use crate::lob::order::OrderContainer;

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
            fill_events.push(FillEvent::new(order_id, taken));

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

#[cfg(test)]
mod tests {
    use crate::lob::book::{Book, Side};
    use crate::lob::sequencer::{AtomicMonotonicSequencer, OrderSequencer};

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
        assert_eq!(fill_events.len(), 1);
        assert_eq!(left_after_taking, 0.0);
        assert_eq!(pl.num_orders_in_queue(), 2);
        assert_eq!(pl.volume, 250.0);

        pl.take(50.0);
        assert_eq!(fill_events.len(), 1);
        assert_eq!(left_after_taking, 0.0);
        assert_eq!(pl.num_orders_in_queue(), 1);
        assert_eq!(pl.volume, 200.0);

        let (fill_events, left_after_taking) = pl.take(250.0);
        assert_eq!(fill_events.len(), 1);
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
        assert_eq!(fill_events.len(), 1);
        assert_eq!(pl.num_orders_in_queue(), 0);
    }

    #[test]
    fn test_price_levels_buy_make() {
        let mut pls = Book::new(Side::Buy);
        let seq = AtomicMonotonicSequencer::new();

        pls.make(10.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(11.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(15.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(12.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(12.0, OrderContainer::new(100.0, seq.next_order_id()));

        assert_eq!(pls.depth(), 4);
        assert_eq!(pls.volume(), 500.0);
        assert_eq!(pls.top(), Some(15.0));
    }

    #[test]
    fn test_price_levels_buy_take() {
        let mut pls = Book::new(Side::Buy);
        let seq = AtomicMonotonicSequencer::new();

        pls.make(10.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(11.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(15.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(12.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(12.0, OrderContainer::new(100.0, seq.next_order_id()));

        assert_eq!(pls.top(), Some(15.0));
        assert_eq!(pls.depth(), 4);
        assert_eq!(pls.volume(), 500.0);

        let fill_events = pls.take(250.0).unwrap();

        assert_eq!(fill_events.len(), 3);
        assert_eq!(pls.volume(), 250.0);
        assert_eq!(pls.top(), Some(12.0));
        assert_eq!(pls.depth(), 3);
    }

    #[test]
    fn test_price_levels_sell_make() {
        let mut pls = Book::new(Side::Sell);
        let seq = AtomicMonotonicSequencer::new();

        pls.make(10.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(11.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(15.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(12.0, OrderContainer::new(100.0, seq.next_order_id()));
        pls.make(12.0, OrderContainer::new(100.0, seq.next_order_id()));

        assert_eq!(pls.depth(), 4);
        assert_eq!(pls.volume(), 500.0);
        assert_eq!(pls.top(), Some(10.0));
    }
}
