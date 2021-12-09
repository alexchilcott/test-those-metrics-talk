In-memory Jaeger collector designed for in-process component tests.

This crate provides a mock Jaeger collector that can be used for testing. The main entry point is [`DetachedJaegerCollectorServer::start()`], which starts a server in a separate thread (which is then detached and will live until the process terminates) on an available port allocated by the operating system.

While this may seem unusual, it is useful for testing components that make use of globally-registered telemetry services, where the telemetry registration is also performed on a per-process rather than per-thread basis.
