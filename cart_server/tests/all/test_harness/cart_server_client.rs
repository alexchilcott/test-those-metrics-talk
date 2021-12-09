use anyhow::Context;
use prometheus_parse::Scrape;
use reqwest::Response;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde_json::json;
use tracing::instrument;

pub struct CartServerClient {
    client: ClientWithMiddleware,
    host: String,
    port: u16,
}

impl CartServerClient {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        let client = ClientBuilder::new(
            reqwest::ClientBuilder::new()
                .build()
                .expect("Failed to build http client"),
        )
        .with(TracingMiddleware)
        .build();
        CartServerClient {
            client,
            host: host.into(),
            port,
        }
    }

    /// Builds a URL to a relative path hosted by our service
    fn build_url(&self, relative_path: impl Into<String>) -> String {
        format!("http://{}:{}{}", self.host, self.port, relative_path.into())
    }

    #[instrument(skip(self))]
    pub async fn send_add_item_to_cart_request(
        &self,
        item_id: &str,
    ) -> Result<Response, anyhow::Error> {
        self.client
            .post(self.build_url("/items"))
            .json(&json!({ "item_id": item_id }))
            .send()
            .await
            .context("Failed to make request to server")
    }

    #[instrument(skip(self))]
    pub async fn get_metrics(&self) -> Result<Scrape, anyhow::Error> {
        let response = self
            .client
            .get(self.build_url("/metrics"))
            .send()
            .await
            .context("Failed to send request to server")?
            .error_for_status()
            .context("Server returned an error status code")?;

        let text = response.text().await.context("Failed to read body")?;

        let lines = text.lines().map(|line| Ok(line.to_owned()));

        let parsed_scrape = Scrape::parse(lines).context("Failed to parse scrape from response")?;
        Ok(parsed_scrape)
    }
}
