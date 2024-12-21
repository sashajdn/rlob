pub mod http;

use async_trait::async_trait;

use crate::engine::trading_engine::TradingEngine;
use std::sync::Arc;

#[async_trait]
pub trait TradingServer<T>: Sync + Send
where
    T: TradingEngine + Sync + Send + 'static,
{
    fn run(
        &self,
        trading_engine: Arc<T>,
    ) -> impl std::future::Future<Output = std::io::Result<()>> + Send;
}
