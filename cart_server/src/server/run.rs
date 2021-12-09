use crate::data_sources::stock_service::StockServiceClient;
use crate::Configuration;
use actix_web::dev::Server;
use actix_web::web::{post, Data};
use actix_web::{App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use anyhow::Context;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use super::add_item_to_cart_route;

pub async fn run_server(
    config: Configuration,
    listener: TcpListener,
) -> Result<Server, anyhow::Error> {
    let client = ClientBuilder::new(
        reqwest::ClientBuilder::new()
            .build()
            .context("Failed to build http client")?,
    )
    .with(TracingMiddleware)
    .build();

    let stock_service_client = Data::new(StockServiceClient::new(config.stock_service_url, client));

    let prometheus = PrometheusMetricsBuilder::new("")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let server = HttpServer::new(move || {
        App::new()
            // The /metrics endpoint here is hosted on the same server as the
            // /items endpoint. For publicly-accessible APIs, we would likely want
            // to host the /metrics endpoint on a separate server.
            .wrap(prometheus.clone())
            .wrap(TracingLogger::default())
            .app_data(stock_service_client.clone())
            .route("/items", post().to(add_item_to_cart_route::handler))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
