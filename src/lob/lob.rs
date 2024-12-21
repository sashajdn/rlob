use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::lob::book::Book;
use crate::lob::book::Side;
use crate::lob::order::{OrderContainer, OrderRequest};
use crate::lob::sequencer::{OrderID, OrderSequencer};

pub struct LimitOrderBook<S: OrderSequencer> {
    bids: Arc<Mutex<Book>>,
    asks: Arc<Mutex<Book>>,
    order_sequencer: Arc<S>,
}

pub struct OrderResult {
    order_id: OrderID,
}

impl<S: OrderSequencer> LimitOrderBook<S> {
    pub fn new(order_sequencer: Arc<S>) -> Self {
        Self {
            bids: Arc::new(Mutex::new(Book::new(Side::Buy))),
            asks: Arc::new(Mutex::new(Book::new(Side::Sell))),
            order_sequencer,
        }
    }

    pub fn place_order(&mut self, order: OrderRequest) -> Result<OrderResult, PlaceOrderError> {
        let order_id = self.order_sequencer.next_order_id();

        match order {
            OrderRequest::Limit(params) => {
                let order_container = OrderContainer::new(params.quantity, order_id);

                let book = match params.side {
                    Side::Buy => &self.bids,
                    Side::Sell => &self.asks,
                };

                let mut book = book.lock().expect("BUG: failed to take lock");
                book.make(params.limit_price, order_container);
            }
            OrderRequest::Market(params) => {
                let order_container = OrderContainer::new(params.quantity, order_id);

                let book = match params.side {
                    Side::Buy => &self.asks,
                    Side::Sell => &self.bids,
                };

                let mut book = book.lock().expect("BUG: failed to take lock");
                let _ = book.take(order_container.size); // TODO: properly handle the errors upstream
            }
        }

        Ok(OrderResult { order_id })
    }
}

#[derive(Error, Debug)]
pub enum PlaceOrderError {
    #[error("time in force applied")]
    TimeInForceError,
    #[error("crosses the spread")]
    CrossedSpreadError,
}
