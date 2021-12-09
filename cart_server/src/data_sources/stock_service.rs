use anyhow::Context;
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;

pub struct StockServiceClient {
    client: ClientWithMiddleware,
    base_url: String,
}

impl StockServiceClient {
    pub fn new(base_url: String, client: ClientWithMiddleware) -> Self {
        Self { client, base_url }
    }

    pub async fn get_stock(&self, item_id: &str) -> Result<u32, anyhow::Error> {
        #[derive(Deserialize)]
        struct ResponseModel {
            pub available_stock: u32,
        }

        let response = self
            .client
            .get(format!("{}/stock/{}", self.base_url, item_id))
            .send()
            .await
            .context("Failed to make request")?
            .error_for_status()
            .context("Error response returned")?
            .json::<ResponseModel>()
            .await
            .context("Invalid response returned")?;

        Ok(response.available_stock)
    }
}
