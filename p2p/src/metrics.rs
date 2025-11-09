// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

use anyhow::Result;
use hyper::{
    body::Body,
    service::{make_service_fn, service_fn},
    Response, Server, StatusCode,
};
use log::{info, warn};
use once_cell::sync::Lazy;
use prometheus::{Encoder, IntCounter, IntGauge, Registry, TextEncoder};
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
use tokio::task::JoinHandle;

static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);
static HANDSHAKES_ATTEMPTED: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new(
        "handshakes_attempted_total",
        "Total number of inbound or outbound handshake attempts",
    )
    .expect("create counter");
    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("register counter");
    counter
});

static HANDSHAKES_SUCCESS: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new(
        "handshakes_success_total",
        "Total number of successful handshakes",
    )
    .expect("create counter");
    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("register counter");
    counter
});

static HANDSHAKES_FAILED: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new(
        "handshakes_failed_total",
        "Total number of failed handshakes",
    )
    .expect("create counter");
    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("register counter");
    counter
});

static ROTATIONS_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let counter = IntCounter::new(
        "rotations_total",
        "Total number of ML-KEM key rotations performed",
    )
    .expect("create counter");
    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("register counter");
    counter
});

static CURRENT_PEERS: Lazy<IntGauge> = Lazy::new(|| {
    let gauge = IntGauge::new(
        "current_peers",
        "Current number of peers with at least one established connection",
    )
    .expect("create gauge");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("register gauge");
    gauge
});

static SERVER_STARTED: AtomicBool = AtomicBool::new(false);
static SWARM_READY: AtomicBool = AtomicBool::new(false);
static _SERVER_HANDLE: Lazy<Mutex<Option<JoinHandle<()>>>> = Lazy::new(|| Mutex::new(None));

pub fn spawn_metrics_server(addr: SocketAddr) -> Result<()> {
    if SERVER_STARTED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(());
    }

    info!("Starting metrics server on {addr}");

    let handle = tokio::spawn(async move {
        let make_svc = make_service_fn(|_| async {
            Ok::<_, Infallible>(service_fn(|req| async move {
                let path = req.uri().path();
                let response = match path {
                    "/metrics" => metrics_response(),
                    "/healthz" => health_response(),
                    _ => Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::from("not found\n"))
                        .expect("response"),
                };
                Ok::<_, Infallible>(response)
            }))
        });

        if let Err(err) = Server::bind(&addr).serve(make_svc).await {
            warn!("metrics server stopped: {err}");
        }
    });

    *_SERVER_HANDLE
        .lock()
        .expect("metrics server handle poisoned") = Some(handle);

    Ok(())
}

pub fn mark_swarm_ready() {
    SWARM_READY.store(true, Ordering::SeqCst);
}

pub fn record_handshake_attempt() {
    HANDSHAKES_ATTEMPTED.inc();
}

pub fn record_handshake_success() {
    HANDSHAKES_SUCCESS.inc();
}

pub fn record_handshake_failure() {
    HANDSHAKES_FAILED.inc();
}

pub fn record_rotation() {
    ROTATIONS_TOTAL.inc();
}

pub fn set_current_peers(count: usize) {
    CURRENT_PEERS.set(count as i64);
}

fn metrics_response() -> Response<Body> {
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    if let Err(err) = encoder.encode(&metric_families, &mut buffer) {
        warn!("failed to encode prometheus metrics: {err}");
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("failed to encode metrics\n"))
            .expect("response");
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", encoder.format_type())
        .body(Body::from(buffer))
        .expect("response")
}

fn health_response() -> Response<Body> {
    if SWARM_READY.load(Ordering::SeqCst) {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("ok\n"))
            .expect("response")
    } else {
        Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("initializing\n"))
            .expect("response")
    }
}
