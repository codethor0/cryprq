use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use regex::Regex;

pub async fn spawn_metrics_endpoint(addr: SocketAddr, metric: String) {
    let listener = TcpListener::bind(addr).await.unwrap();
    let metric = Arc::new(Mutex::new(metric));
    tokio::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
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
        let addr: SocketAddr = "127.0.0.1:9090".parse().unwrap();
        spawn_metrics_endpoint(addr, metric.clone()).await;
        sleep(std::time::Duration::from_millis(100)).await;
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let mut buf = vec![0u8; 128];
        let n = stream.read(&mut buf).await.unwrap();
        let text = String::from_utf8_lossy(&buf[..n]);
        let re = Regex::new(r"cryprq_keyburn_proof\{anon_id=\\"[a-zA-Z0-9]+\\"\} [a-f0-9]+").unwrap();
        assert!(re.is_match(&text));
    }
}
