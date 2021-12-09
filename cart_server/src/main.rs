use std::net::TcpListener;

use anyhow::Context;
use cart_server::{initialise_tracing, run_server, Configuration};

fn load_config() -> Result<Configuration, anyhow::Error> {
    Ok(Configuration {
        host: "127.0.0.1".into(),
        port: 12345,
        stock_service_url: "https://stock-server".into(),
        collector_url: "http://127.0.0.1:14268".into(),
    })
}

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = load_config().context("Failed to load server configuration")?;
    initialise_tracing(config.collector_url.as_str());
    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&address).context(format!("Failed to bind to {}", address))?;
    run_server(config, listener)
        .await
        .context("Failed to build server")?
        .await
        .context("Server terminated unexpectedly")?;
    Ok(())
}
