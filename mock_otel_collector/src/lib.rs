#![doc = include_str!("../README.md")]

pub mod jaeger_models;
mod server;
pub use server::DetachedMockOtelCollector;
