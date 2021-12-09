use std::io;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::{self};

use actix_web::dev::Server;
use actix_web::rt::System;
use actix_web::web::{get, post, BytesMut, Data, Payload};
use actix_web::{App, HttpResponse, HttpServer, Responder};
use anyhow::Context;
use futures_util::StreamExt;
use rctree::Node;
use reqwest::ClientBuilder;
use thrift::protocol::TBinaryInputProtocol;

use crate::jaeger_models::span_tree::build_span_tree;
use crate::jaeger_models::{Batch, Span};

async fn post_traces_handler(
    payload: Payload,
    received_batches: Data<Mutex<Vec<Batch>>>,
) -> impl Responder {
    async fn handle(
        mut payload: Payload,
        received_batches: Data<Mutex<Vec<Batch>>>,
    ) -> Result<(), anyhow::Error> {
        let mut bytes = BytesMut::new();
        while let Some(item) = payload.next().await {
            bytes.extend_from_slice(&item?);
        }

        let mut binary_input = TBinaryInputProtocol::new(bytes.as_ref(), false);
        let batch = Batch::read_from_in_protocol(&mut binary_input)?;
        let mut data = received_batches.lock().unwrap();
        data.push(batch);
        Ok(())
    }

    match handle(payload, received_batches).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    }
}

fn run_server(
    listener: TcpListener,
    batch_store: Arc<Mutex<Vec<Batch>>>,
) -> Result<Server, io::Error> {
    let batch_store = Data::from(batch_store);

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(batch_store.clone())
            .route("/up", get().to(HttpResponse::Ok))
            .route("/api/traces", post().to(post_traces_handler))
    })
    .listen(listener)?
    .run())
}

pub struct DetachedMockOtelCollector {
    base_url: String,
    batch_store: Arc<Mutex<Vec<Batch>>>,
}

impl DetachedMockOtelCollector {
    /// Start a new detached Jaeger collector server, listening on a randomly allocated port.
    /// This server runs on a dedicated thread with its own runtime, rather than simply in its
    /// own task inside the current runtime. This allows it to be started once from within a
    /// the current runtime, while avoiding it being shut down when the main runtime is dropped.
    ///
    /// This server is not intended to be used in production, but rather as a mock for testing.
    pub fn start() -> Result<Self, anyhow::Error> {
        let address = "127.0.0.1:0";
        let listener =
            TcpListener::bind(address).with_context(|| format!("Failed to bind to {}", address))?;

        let batch_store = Arc::new(Mutex::new(Vec::<Batch>::new()));
        let base_url = format!("http://127.0.0.1:{}", listener.local_addr()?.port());

        let thread_batch_store = batch_store.clone();
        thread::spawn(move || {
            System::new().block_on(async move {
                run_server(listener, thread_batch_store)
                    .expect("Failed to listen for incoming connections")
                    .await
                    .expect("Server failed unexpectedly");
                panic!("Server terminated unexpectedly");
            });
        });

        Ok(Self {
            base_url,
            batch_store,
        })
    }

    /// Test whether the server has started successfully.
    pub async fn ping(&self) -> Result<(), anyhow::Error> {
        let reqwest_client = ClientBuilder::new()
            .build()
            .context("Failed to build reqwest client")?;

        let url = format!("{}/up", self.base_url());
        let _ = reqwest_client.get(url).send().await?.error_for_status()?;
        Ok(())
    }

    /// Get the base URL of the server.
    pub fn base_url(&self) -> String {
        self.base_url.to_owned()
    }

    /// Retrieve a trace, in the form of a [`rctree::Node<Span>`], from the in-memory
    /// store of received [`Span`]s.
    pub async fn get_trace(&self, trace_id: &str) -> Result<Node<Span>, anyhow::Error> {
        let spans: Vec<Span> = {
            let batches = self.batch_store.lock().unwrap();
            batches
                .iter()
                .flat_map(|batch| batch.spans.iter())
                .filter(|x| x.hex_trace_id() == trace_id)
                .cloned()
                .collect()
        };

        build_span_tree(spans)
    }
}
