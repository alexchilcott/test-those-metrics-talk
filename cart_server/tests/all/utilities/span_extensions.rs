use opentelemetry::trace::TraceContextExt;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub trait SpanExt {
    fn otel_trace_id(&self) -> String;
}

impl SpanExt for Span {
    fn otel_trace_id(&self) -> String {
        self.context().span().span_context().trace_id().to_hex()
    }
}
