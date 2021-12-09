use actix_web::{web, HttpResponse, Responder};
use anyhow::Context;
use serde::Deserialize;
use tracing::instrument;

use crate::data_sources::stock_service::StockServiceClient;

#[derive(Debug, Deserialize)]
pub struct Body {
    pub item_id: String,
}

#[instrument(skip(stock_service_client))]
async fn add_item_to_cart(
    item_id: &str,
    stock_service_client: &StockServiceClient,
) -> Result<(), anyhow::Error> {
    let stock_count = stock_service_client
        .get_stock(item_id)
        .await
        .context("Failed to get stock")?;

    if stock_count == 0 {
        return Err(anyhow::anyhow!("No stock available"));
    }

    Ok(())
}

#[instrument(skip(stock_service_client))]
pub async fn handler(
    body: web::Json<Body>,
    stock_service_client: web::Data<StockServiceClient>,
) -> impl Responder {
    add_item_to_cart(&body.item_id, &stock_service_client)
        .await
        .map_or_else(
            |err| {
                dbg!(err);
                HttpResponse::InternalServerError().finish()
            },
            |()| HttpResponse::Ok().finish(),
        )
        .await
}
