use super::order::OrderContainer;

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
        self.orders.push(order);
    }

    #[inline]
    pub fn take(&mut self, qty: f64) -> (Vec<FillEvent>, f64) {
        let mut order_idx_to_drain_upto: usize = 0;
        let mut taken_total = 0.0;
        let mut remaining_qty = qty;

        let mut fill_events: Vec<FillEvent> = Vec::with_capacity(self.orders.len());

        for order in &mut self.orders {
            let (left, order_remaining_size, taken) = order.take_qty(remaining_qty);
            remaining_qty = left;

            if taken == 0.0 {
                break;
            }

            taken_total += taken;

            let order_id = std::mem::take(&mut order.order_id);
            fill_events.push(FillEvent {
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
        (fill_events, qty - taken_total)
    }
}

pub struct FillEvent {
    order_id: String,
    size: f64,
}
