pub mod http;

use crate::engine::trading_engine::{self, TradingEngine};
use std::sync::Arc;

pub trait TradingServer<T>: Sync + Send
where
    T: trading_engine::TradingEngine + 'static,
{
    fn run(
        &self,
        trading_engine: Arc<T>,
    ) -> impl std::future::Future<Output = std::io::Result<()>> + Send;
}
