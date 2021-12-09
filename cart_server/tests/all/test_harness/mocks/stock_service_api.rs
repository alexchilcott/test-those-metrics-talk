use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub struct MockStockServiceApi(MockServer);

impl MockStockServiceApi {
    pub async fn new() -> Self {
        Self(MockServer::builder().start().await)
    }

    pub fn base_url(&self) -> String {
        self.0.uri()
    }

    pub async fn set_stock_count(&self, item_id: &str, count: u32) {
        Mock::given(method("GET"))
            .and(path(format!("/stock/{}", item_id)))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({ "available_stock": count })),
            )
            .mount(&self.0)
            .await;
    }

    pub async fn setup_failure(&self, item_id: &str) {
        Mock::given(method("GET"))
            .and(path(format!("/stock/{}", item_id)))
            .respond_with(ResponseTemplate::new(500))
            .mount(&self.0)
            .await;
    }
}
