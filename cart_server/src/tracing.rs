use opentelemetry::sdk::export::trace::SpanExporter;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::trace::{Config, Sampler, TracerProvider};
use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::{layer::SubscriberExt, Registry};

pub const SERVER_NAME: &str = "cart_server";

pub fn initialise_tracing(collector_url: &str) {
    let span_exporter = build_jaeger_exporter(collector_url);
    let tracer_provider = build_otel_tracer_provider(span_exporter);
    let tracer = tracer_provider.get_tracer(SERVER_NAME, None);
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default()
        .with(filter_fn(|span| {
            // Removing some noise from our traces
            span.name() != "parse_headers" && span.name() != "encode_headers"
        }))
        .with(otel_layer);

    opentelemetry::global::set_tracer_provider(tracer_provider);
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}

fn build_otel_tracer_provider<E: SpanExporter + 'static>(span_exporter: E) -> TracerProvider {
    TracerProvider::builder()
        .with_batch_exporter(span_exporter, opentelemetry::runtime::Tokio)
        .with_config(Config {
            sampler: Box::new(Sampler::AlwaysOn),
            ..Default::default()
        })
        .build()
}

fn build_jaeger_exporter(collector_url: &str) -> opentelemetry_jaeger::Exporter {
    opentelemetry_jaeger::new_pipeline()
        .with_collector_endpoint(format!("{}/api/traces", collector_url))
        .with_service_name(SERVER_NAME)
        .init_exporter()
        .expect("Failed to build Jaeger span exporter")
}
