pub mod cart_server_client;
pub mod mocks;
use crate::utilities::retry_loop::{self, retry_until_ok, RetryTimeoutError};

use self::cart_server_client::CartServerClient;
use self::mocks::MockStockServiceApi;
use actix_rt::task::JoinHandle;
use actix_rt::System;
use actix_web::dev::Server;
use anyhow::Error;
use cart_server::{initialise_tracing, run_server, Configuration};
use mock_otel_collector::jaeger_models::Span;
use mock_otel_collector::DetachedMockOtelCollector;
use opentelemetry::global::force_flush_tracer_provider;
use rctree::Node;
use std::future::pending;
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::sync::OnceCell;

static TRACING_INIT: OnceCell<DetachedMockOtelCollector> = OnceCell::const_new();

/// Initialising the telemetry collection for these tests requires a bit of a ballet.
///
/// Since the `tracing` crate and the `opentelemetry` crate rely quite heavily on global
/// state, we are required to configure this in our tests exactly once. This function,
/// however, will be called multiple times; once per test.
///
/// The first time it is called, it will initialise the global `tracing` and
/// `opentelemetry` state, in the same way as would happen during our application
/// start up. It will also create a single [`DetachedMockOtelCollector`] instance
/// to receive traces and store them in memory, for verification from tests.
///
/// Subsequent calls will do nothing other than return the stored reference to the
/// previously started DetachedMockOtelCollector.
async fn initialise_telemetry_collection() -> &'static DetachedMockOtelCollector {
    // This [`OnceCell`] guarantees the initialisation logic will be called only once, even
    // if this function is called multiple times.
    let server = TRACING_INIT
        .get_or_init(|| async {
            // Create the DetachedMockOtelCollector to receive traces from our application.
            let detached_otel_collector_server =
                DetachedMockOtelCollector::start().expect("Failed to start otel collector");

            // Wait for the DetachedMockOtelCollector to be ready to accept
            // connections.
            retry_loop::retry_until_ok(
                || async { detached_otel_collector_server.ping().await },
                Duration::from_secs(10),
                Duration::from_secs(1),
                Duration::from_millis(50),
            )
            .await
            .expect("Timed out waiting for otel collector to become ready");

            // `opentelemetry` requires a running async runtime exists that it can
            // use to spawn tasks. However, when running tests, each test is run
            // in its own actix `System` which is stopped upon completion of the test.
            //
            // Since the `opentelemetry` state is global, we need to avoid it capturing
            // a reference to a short lived `System` that will be stopped when the test
            // that created it completes.
            //
            // To do that, we spawn a new thread (which we detach, so it lives until
            // the process terminates), and create a new `System` within that thread.
            // We then perform our applications initialisation logic inside the new
            // `System`. We use a `mpsc::channel()` to communicate back to the thread
            // executing the initialisation logic when it is safe to proceed.
            let (sender, receiver) = mpsc::channel();
            let collector_url = detached_otel_collector_server.base_url();
            thread::spawn(move || {
                System::new().block_on(async move {
                    initialise_tracing(&collector_url);
                    sender.send(()).unwrap();
                    pending::<()>().await
                })
            });
            receiver.recv().unwrap();

            // Finally, we return the created otel collector server.
            detached_otel_collector_server
        })
        .await;

    server
}

pub struct TestHarness {
    /// A client to use to interact with our service
    pub client: CartServerClient,

    /// The configuration that this instance of the
    /// service started with.
    pub config: Configuration,

    /// The mock stock service API.
    pub mock_stock_service: MockStockServiceApi,

    /// A reference to the shared otel trace collector.
    /// It should be noted that this is static and shared
    /// across tests. Tests should therefore only be
    /// querying for specific traces they produce.
    mock_otel_collector: &'static DetachedMockOtelCollector,

    /// A join handle for the server thread.
    _server_join_handle: JoinHandle<Result<Server, Error>>,
}

impl TestHarness {
    /// Starts a new instance of the service and any
    /// required mocks, returning a `TestHarness` which can
    /// be used to interact with the service and the mocks.
    pub async fn start() -> TestHarness {
        // Create the mock otel collector, and configure
        // the telemetry crate to send traces to it.
        let mock_otel_collector = initialise_telemetry_collection().await;

        // Start a mock "stock service" on a random port
        let mock_stock_service = MockStockServiceApi::new().await;

        // Allocate a port for the service to listen on
        let host = "127.0.0.1";
        let listener = TcpListener::bind(format!("{}:0", host)).unwrap();

        // Build a struct containing all our service's
        // configuration
        let config = Configuration {
            host: host.into(),
            port: listener.local_addr().unwrap().port(),
            stock_service_url: mock_stock_service.base_url(),
            collector_url: mock_otel_collector.base_url(),
        };

        // Start the service on a background thread
        let server = run_server(config.clone(), listener);
        let server_join_handle = actix_rt::spawn(server);

        // Create a client to use to interact with the
        // service
        let client = CartServerClient::new("127.0.0.1", config.port);

        // Bundle it all together
        TestHarness {
            client,
            config,
            mock_stock_service,
            mock_otel_collector,
            _server_join_handle: server_join_handle,
        }
    }

    pub async fn check_trace<F>(
        &self,
        trace_id: String,
        check_trace: F,
    ) -> Result<(), RetryTimeoutError<anyhow::Error>>
    where
        F: Fn(Node<Span>) -> Result<(), anyhow::Error>,
    {
        let timeout = Duration::from_secs(5);
        retry_until_ok(
            || async {
                // Since our telemetry state is global and shared between our
                // test and our server, we can cheat a little here and force
                // `opentelemetry` to flush any pending traces
                force_flush_tracer_provider();
                let trace = self.mock_otel_collector.get_trace(&trace_id).await?;
                check_trace(trace)
            },
            timeout,
            timeout,
            Duration::from_millis(100),
        )
        .await
    }
}
