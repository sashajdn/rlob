use crate::lob::order::OrderContainer;
use crate::lob::pricelevel::PriceLevel;
use crate::lob::sequencer::OrderID;

use thiserror::Error;

const DEFAULT_PRICE_LEVELS_INITIAL_CAPACITY: usize = 2 << 10; // 2048

pub struct Book {
    levels: Vec<PriceLevel>,
    side: Side,
    volume: f64,
}

// TODO: this should be the only place where we have to implement the logic for book.
impl Book {
    pub fn new(side: Side) -> Self {
        Self {
            levels: Vec::with_capacity(DEFAULT_PRICE_LEVELS_INITIAL_CAPACITY),
            side,
            volume: 0.0,
        }
    }

    #[inline]
    pub fn make(&mut self, price: f64, order: OrderContainer) {
        self.volume += order.size;

        if let Some(price_level) = self.levels.iter_mut().find(|level| level.price == price) {
            price_level.make(order);
        } else {
            let mut price_level = PriceLevel::new(price, 0.0);
            price_level.make(order);
            self.levels.push(price_level);
        }

        self.levels.sort_by(self.side.price_levels_comparator());
    }

    #[inline]
    pub fn take(&mut self, qty: f64) -> Result<FillEvents, TakeError> {
        // validate the take request.
        self.validate_take(qty)?;
        self.volume -= qty;

        let mut total_fill_events = FillEvents::new(32);
        let mut remaining_qty = qty;

        for pl in &mut self.levels {
            let (fill_events, remaining_qty_after_take) = pl.take(remaining_qty);
            total_fill_events.0.extend(fill_events.0);
            remaining_qty = remaining_qty_after_take;

            if remaining_qty == 0.0 {
                break;
            }
        }

        self.levels.retain(|pl| pl.volume > 0.0);
        Ok(total_fill_events)
    }

    #[inline]
    pub fn depth(&self) -> usize {
        self.levels.len()
    }

    #[inline]
    pub fn volume(&self) -> f64 {
        self.volume
    }

    #[inline]
    pub fn top(&self) -> Option<f64> {
        self.levels.first().map(|pl| pl.price)
    }

    // TODO: remove
    pub fn print_price_levels(&self) {
        for pl in &self.levels {
            println!("Price: {}, Volume: {}", pl.price, pl.volume);
        }
    }

    #[inline]
    fn validate_take(&self, qty: f64) -> Result<(), TakeError> {
        if qty > self.volume {
            Err(TakeError::NotEnoughVolume)
        } else if qty == 0.0 {
            Err(TakeError::ZeroQuantity)
        } else if self.levels.is_empty() {
            Err(TakeError::EmptyBook)
        } else {
            Ok(())
        }
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
            Side::Sell => |a, b| a.price.total_cmp(&b.price),
            Side::Buy => |a, b| b.price.total_cmp(&a.price),
        }
    }
}

pub struct FillEvents(Vec<FillEvent>);

impl FillEvents {
    pub fn new(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, fill_event: FillEvent) {
        self.0.push(fill_event);
    }
}

pub struct FillEvent {
    order_id: OrderID,
    size: f64,
}

impl FillEvent {
    pub fn new(order_id: OrderID, size: f64) -> Self {
        Self { order_id, size }
    }
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
mod test {
    use super::*;
    use crate::lob::sequencer::{AtomicMonotonicSequencer, OrderSequencer};

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
        assert_eq!(pls.volume, 500.0);
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
        assert_eq!(pls.volume, 500.0);

        let fill_events = pls.take(250.0).unwrap();

        assert_eq!(fill_events.0.len(), 3);
        assert_eq!(pls.volume, 250.0);
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
        assert_eq!(pls.volume, 500.0);
        assert_eq!(pls.top(), Some(10.0));
    }
}
