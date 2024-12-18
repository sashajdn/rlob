use std::sync::atomic::{AtomicU64, Ordering};

pub type OrderID = u64;

pub trait OrderSequencer: Send + Sync {
    fn next_order_id(&self) -> OrderID;
}

pub struct AtomicMonotonicSequencer {
    counter: AtomicU64,
}

impl AtomicMonotonicSequencer {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1), // count from 1.
        }
    }
}

impl OrderSequencer for AtomicMonotonicSequencer {
    fn next_order_id(&self) -> OrderID {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}
