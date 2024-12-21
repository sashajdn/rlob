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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_atomic_monotonic_sequencer_sanity_check() {
        let sequencer = Arc::new(AtomicMonotonicSequencer::new());
        let threads: Vec<_> = (0..10)
            .map(|_| {
                let sequencer = sequencer.clone();
                std::thread::spawn(move || {
                    for _ in 0..10 {
                        sequencer.next_order_id();
                    }
                })
            })
            .collect();

        threads.into_iter().for_each(|t| t.join().unwrap());

        assert_eq!(sequencer.next_order_id(), 101);
    }
}
