# Test Those Metrics!

The code in this repo accompanies a talk given at the [Rust London TrueLayer Takeover](https://skillsmatter.com/meetups/13657-rust-ldn-dec21) in December 2021, as a demonstration of one way the observability of a microservice can be exercised and verified during automated tests.

Disclaimer: This code is provided AS IS. It is NOT intended as an example of a production-ready service.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install), version >= `1.56.1`.

## To Test

Run `cargo test`. No additional services are assumed to be running.

## Layout

This repository contains a cargo workspace consisting of two crates:

- `cart_server`, the main binary, that hosts our HTTP server;
- `mock_jaeger_collector`, a library crate consisting of a mock Jaeger collector service, for use in black box testing the cart server.
