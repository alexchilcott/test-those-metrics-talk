use crate::test_harness::TestHarness;
use crate::utilities::tracer::Tracer;
use anyhow::{anyhow, Context};
use mock_otel_collector::jaeger_models::{Span, TagValue};
use prometheus_parse::Value;
use rctree::Node;
use reqwest::StatusCode;
use uuid::Uuid;

#[actix_rt::test]
pub async fn add_item_when_stock_for_item_exists_returns_ok() {
    // Arrange
    let test_harness = TestHarness::start().await;
    let item_id = Uuid::new_v4().to_string();
    test_harness
        .mock_stock_service
        .set_stock_count(&item_id, 100)
        .await;

    // Act
    let (response, trace_id) = Tracer::trace(async {
        test_harness
            .client
            .send_add_item_to_cart_request(&item_id)
            .await
            .expect("Failed to send request")
    })
    .await;

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let metrics = test_harness
        .client
        .get_metrics()
        .await
        .expect("Failed to get metrics");
    let http_requests_total_sample = metrics
        .samples
        .iter()
        .find(|sample| {
            sample.metric == "http_requests_total"
                && sample.labels.get("endpoint") == Some("/items")
                && sample.labels.get("method") == Some("POST")
                && sample.labels.get("status") == Some("200")
        })
        .expect(r#"No matching http_requests_total sample found"#);

    assert_eq!(http_requests_total_sample.value, Value::Counter(1.into()));

    test_harness
        .check_trace(trace_id, |trace| {
            let stock_check_request_span = trace
                .descendants()
                .find(|s| s.borrow().operation_name == format!("GET /stock/{}", &item_id))
                .ok_or_else(|| anyhow!(r#"No span found for stock check request""#))?;

            check_tag(
                &stock_check_request_span,
                "http.method",
                TagValue::String("GET"),
            )
            .context("http.method tag was not correct")?;

            check_tag(
                &stock_check_request_span,
                "http.status_code",
                TagValue::Long(200),
            )
            .context("http.status_code tag was not correct")
        })
        .await
        .expect("Expected trace was not available within timeout");
}

#[actix_rt::test]
pub async fn add_item_when_stock_service_is_unavailable_returns_error() {
    // Arrange
    let test_harness = TestHarness::start().await;
    let item_id = Uuid::new_v4().to_string();
    test_harness
        .mock_stock_service
        .setup_failure(&item_id)
        .await;

    // Act
    let (response, trace_id) = Tracer::trace(async {
        test_harness
            .client
            .send_add_item_to_cart_request(&item_id)
            .await
            .expect("Failed to send request")
    })
    .await;

    // Assert
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let metrics = test_harness
        .client
        .get_metrics()
        .await
        .expect("Failed to get metrics");
    let http_requests_total_sample = metrics
        .samples
        .iter()
        .find(|sample| {
            sample.metric == "http_requests_total"
                && sample.labels.get("endpoint") == Some("/items")
                && sample.labels.get("method") == Some("POST")
                && sample.labels.get("status") == Some("500")
        })
        .expect(r#"No matching http_requests_total sample found"#);

    assert_eq!(http_requests_total_sample.value, Value::Counter(1.into()));

    test_harness
        .check_trace(trace_id, |trace| {
            let stock_check_request_span = trace
                .descendants()
                .find(|s| s.borrow().operation_name == format!("GET /stock/{}", &item_id))
                .ok_or_else(|| anyhow!(r#"No span found for stock check request""#))?;

            check_tag(
                &stock_check_request_span,
                "http.method",
                TagValue::String("GET"),
            )
            .context("http.method tag was not correct")?;

            check_tag(
                &stock_check_request_span,
                "http.status_code",
                TagValue::Long(500),
            )
            .context("http.status_code tag was not correct")
        })
        .await
        .expect("Expected trace was not available within timeout");
}

fn check_tag(span: &Node<Span>, key: &str, expected_value: TagValue) -> Result<(), anyhow::Error> {
    let span_ref = span.borrow();
    let tag = span_ref
        .get_tag(key)
        .ok_or_else(|| anyhow!(format!("No tag with key {} was found", key)))?;

    let value = tag.value().context("Could not interpret tag value")?;
    if value == expected_value {
        Ok(())
    } else {
        Err(anyhow!(format!(
            "Tag with key {} was found, but its value was {:?}, not {:?}",
            key, value, expected_value
        )))
    }
}
