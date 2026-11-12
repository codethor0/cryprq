// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use anyhow::Context;
use http_body_util::combinators::BoxBody;
use http_body_util::Full;
use hyper::body::Bytes as HyperBytes;
use hyper::{header::CONTENT_TYPE, service::service_fn, StatusCode};
use hyper_util::rt::TokioIo;
use hyper_util::{rt::TokioExecutor, server::conn::auto::Builder};
use once_cell::sync::Lazy;
use prometheus::{opts, Encoder, Gauge, IntCounter, IntGauge, Registry, TextEncoder};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};
use tokio::net::TcpListener;

// Type aliases to ensure consistent Bytes type (hyper, not libp2p)
type RespBody = BoxBody<HyperBytes, Infallible>;
type Resp = hyper::Response<RespBody>;

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
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind metrics server to {addr}"))?;
    log::info!("event=metrics_server_started addr={addr}");

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .with_context(|| format!("failed to accept connection on {addr}"))?;
        let io = TokioIo::new(stream);
        let service = service_fn(handle_request);

        tokio::task::spawn(async move {
            let builder = Builder::new(TokioExecutor::new());
            if let Err(err) = builder.serve_connection(io, service).await {
                log::warn!("event=metrics_connection_error error={err}");
            }
        });
    }
}

async fn handle_request(req: hyper::Request<hyper::body::Incoming>) -> Result<Resp, Infallible> {
    let response = match req.uri().path() {
        "/metrics" => encode_metrics_response(),
        "/healthz" => health_response(),
        _ => hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(BoxBody::new(Full::new(HyperBytes::from("not found"))))
            .expect("404 response"),
    };

    Ok(response)
}

fn encode_metrics_response() -> Resp {
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("encode metrics");

    let body = BoxBody::new(Full::new(HyperBytes::from(buffer)));
    hyper::Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(body)
        .expect("metrics response")
}

fn health_response() -> Resp {
    if HEALTHY.load(Ordering::Relaxed) {
        let body = BoxBody::new(Full::new(HyperBytes::from("ok")));
        hyper::Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .expect("health ok")
    } else {
        let body = BoxBody::new(Full::new(HyperBytes::from("initializing")));
        hyper::Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(body)
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
