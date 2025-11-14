// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use regex::Regex;

pub async fn spawn_metrics_endpoint(addr: SocketAddr, metric: String) {
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind metrics endpoint at {}: {}", addr, e);
            return;
        }
    };
    let metric = Arc::new(Mutex::new(metric));
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok((s, _)) => s,
                Err(e) => {
                    eprintln!("Failed to accept metrics connection: {}", e);
                    continue;
                }
            };
            let metric = metric.lock().await.clone();
            let _ = socket.write_all(metric.as_bytes()).await;
        }
    });
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::time::sleep;
    use std::net::SocketAddr;
    use regex::Regex;

    #[tokio::test]
    async fn metrics_endpoint_emits_keyburn_proof() {
        let metric = "cryprq_keyburn_proof{anon_id=\"abc123\"} deadbeef".to_string();
        let addr: SocketAddr = "127.0.0.1:9090".parse().expect("valid test address");
        spawn_metrics_endpoint(addr, metric.clone()).await;
        sleep(std::time::Duration::from_millis(100)).await;
        let mut stream = match tokio::net::TcpStream::connect(addr).await {
            Ok(s) => s,
            Err(e) => {
                panic!("Failed to connect to test metrics endpoint: {}", e);
            }
        };
        let mut buf = vec![0u8; 128];
        let n = match stream.read(&mut buf).await {
            Ok(n) => n,
            Err(e) => {
                panic!("Failed to read from test metrics endpoint: {}", e);
            }
        };
        let text = String::from_utf8_lossy(&buf[..n]);
        let re = Regex::new(r"cryprq_keyburn_proof\{anon_id=\\"[a-zA-Z0-9]+\\"\} [a-f0-9]+")
            .expect("valid regex pattern");
        assert!(re.is_match(&text));
    }
}
