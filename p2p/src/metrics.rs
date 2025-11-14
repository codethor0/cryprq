// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

use anyhow::Context;
use hyper::{
    body::Body,
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Response, Server, StatusCode,
};
use once_cell::sync::Lazy;
use prometheus::{opts, Encoder, Gauge, IntCounter, IntGauge, Registry, TextEncoder};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};

static REGISTRY: Lazy<Registry> =
    Lazy::new(|| Registry::new_custom(Some("cryprq".to_string()), None).expect("metrics registry"));

fn register_counter(name: &str, help: &str) -> IntCounter {
    let counter = IntCounter::new(name, help).expect("counter");
    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("register counter");
    counter
}

fn register_gauge(name: &str, help: &str) -> Gauge {
    let gauge = Gauge::with_opts(opts!(name, help)).expect("gauge");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("register gauge");
    gauge
}

fn register_int_gauge(name: &str, help: &str) -> IntGauge {
    let gauge = IntGauge::new(name, help).expect("gauge");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("register gauge");
    gauge
}

static HANDSHAKES_ATTEMPTED: Lazy<IntCounter> =
    Lazy::new(|| register_counter("handshakes_attempted", "Total handshake attempts"));
static HANDSHAKES_SUCCESS: Lazy<IntCounter> =
    Lazy::new(|| register_counter("handshakes_success", "Successful handshakes"));
static HANDSHAKES_FAILED: Lazy<IntCounter> =
    Lazy::new(|| register_counter("handshakes_failed", "Failed handshakes"));

static ROTATIONS_TOTAL: Lazy<IntCounter> =
    Lazy::new(|| register_counter("rotations_total", "Successful key rotations"));
static ROTATION_DURATION_SECONDS: Lazy<Gauge> = Lazy::new(|| {
    register_gauge(
        "rotation_last_duration_seconds",
        "Duration of the last successful rotation",
    )
});
static ROTATION_INTERVAL_SECONDS: Lazy<Gauge> = Lazy::new(|| {
    register_gauge(
        "rotation_interval_seconds",
        "Configured rotation interval in seconds",
    )
});
static ROTATION_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge(
        "rotation_epoch",
        "Current key rotation epoch (8-bit, wraps at 256)",
    )
});
static ACTIVE_PEERS: Lazy<IntGauge> =
    Lazy::new(|| register_int_gauge("current_peers", "Current active peers"));

static HEALTHY: AtomicBool = AtomicBool::new(false);
static ROTATION_COUNTER: AtomicU64 = AtomicU64::new(0);

pub async fn start_metrics_server(addr: SocketAddr) -> anyhow::Result<()> {
    let make_svc = make_service_fn(|_| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);
    log::info!("event=metrics_server_started addr={}", addr);

    server
        .await
        .with_context(|| format!("metrics server failed on {addr}"))
}

async fn handle_request(req: hyper::Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = match req.uri().path() {
        "/metrics" => encode_metrics_response(),
        "/healthz" => health_response(),
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("not found"))
            .expect("404 response"),
    };

    Ok(response)
}

fn encode_metrics_response() -> Response<Body> {
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("encode metrics");

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .expect("metrics response")
}

fn health_response() -> Response<Body> {
    if HEALTHY.load(Ordering::Relaxed) {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("ok"))
            .expect("health ok")
    } else {
        Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("initializing"))
            .expect("health starting")
    }
}

pub(crate) fn mark_swarm_initialized() {
    HEALTHY.store(true, Ordering::Relaxed);
}

pub(crate) fn record_handshake_attempt() {
    HANDSHAKES_ATTEMPTED.inc();
}

pub(crate) fn record_handshake_success() {
    HANDSHAKES_SUCCESS.inc();
}

pub(crate) fn record_handshake_failure() {
    HANDSHAKES_FAILED.inc();
}

pub(crate) fn inc_active_peers() {
    ACTIVE_PEERS.inc();
}

pub(crate) fn dec_active_peers() {
    ACTIVE_PEERS.dec();
}

pub(crate) fn set_rotation_interval(interval: Duration) {
    ROTATION_INTERVAL_SECONDS.set(interval.as_secs_f64());
}

pub(crate) fn record_rotation_success(duration: Duration, epoch: u8) -> u64 {
    let counter = ROTATION_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    ROTATIONS_TOTAL.inc();
    ROTATION_DURATION_SECONDS.set(duration.as_secs_f64());
    ROTATION_EPOCH.set(epoch as i64); // Protocol epoch (u8)
    counter // Return metric counter (u64) for logging compatibility
}
