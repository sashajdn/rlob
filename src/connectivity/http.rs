use crate::connectivity::TradingServer;
use crate::engine::trading_engine::TradingEngine;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;

const MAX_DEFAULT_HTTP_CONNECTIONS: u32 = 100;
const DEFAULT_HTTP_HOST: &'static str = "localhost";
const DEFAULT_HTTP_PORT: u16 = 8080;

struct HTTPTradingServerConfig {
    host: Option<&'static str>,
    port: Option<u16>,
    max_connections: Option<u32>,
}

pub struct HTTPTradingServer {
    host: &'static str,
    port: u16,
    max_connections: u32,
}

impl HTTPTradingServer {
    pub fn new(cfg: HTTPTradingServerConfig) -> HTTPTradingServer {
        HTTPTradingServer {
            port: cfg.port.unwrap_or(DEFAULT_HTTP_PORT),
            host: cfg.host.unwrap_or(DEFAULT_HTTP_HOST),
            max_connections: cfg.max_connections.unwrap_or(MAX_DEFAULT_HTTP_CONNECTIONS),
        }
    }
}

impl<T> TradingServer<T> for HTTPTradingServer
where
    T: TradingEngine + Send + Sync + 'static,
{
    async fn run(&self, trading_engine: Arc<T>) -> std::io::Result<()> {
        let address = format!("{}:{}", self.host, self.port);

        let trading_engine_data = web::Data::new(Arc::clone(&trading_engine));

        HttpServer::new(move || {
            App::new()
                .app_data(trading_engine_data.clone())
                .route("/api/order", web::post().to(place_order::<T>))
        })
        .bind(address)?
        .run()
        .await
    }
}

async fn place_order<T: TradingEngine>(
    _engine: web::Data<Arc<T>>,
    _order: web::Json<OrderRequest>,
) -> impl Responder {
    HttpResponse::Ok().body("Order submitted!")
}

#[derive(serde::Deserialize)]
pub struct OrderRequest {}
