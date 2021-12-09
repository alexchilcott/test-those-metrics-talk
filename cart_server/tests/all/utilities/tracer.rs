use std::future::Future;

use tracing::{instrument, Span};

use super::span_extensions::SpanExt;

pub struct Tracer;

impl Tracer {
    // The instrument attribute opens a span prior to
    // executing the function body and is closed after
    // the function returns.
    #[instrument(skip(f))]
    pub async fn trace<F, T>(f: F) -> (T, String)
    where
        F: Future<Output = T>,
    {
        let trace_id = Span::current().otel_trace_id();
        let result = f.await;
        (result, trace_id)
    }
}
