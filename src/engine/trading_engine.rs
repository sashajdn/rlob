use crate::lob::sequencer::OrderID;

pub struct PlaceOrderResult {
    order_id: OrderID,
}

pub trait TradingEngine: Send + Sync {
    fn place_order_request(&self) -> Result<PlaceOrderResult, ()>;
}
