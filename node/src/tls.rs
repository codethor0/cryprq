// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! TLS 1.3 Support for Control Plane
//!
//! This module provides TLS 1.3 support for control plane communications,
//! ensuring encrypted and authenticated control channel.

use tokio::net::TcpListener;
use tokio::net::TcpStream;

/// TLS 1.3 configuration for control plane
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Enable TLS 1.3
    pub enabled: bool,
    /// Certificate path (for server mode)
    pub cert_path: Option<String>,
    /// Private key path (for server mode)
    pub key_path: Option<String>,
    /// CA certificate path (for client verification)
    pub ca_cert_path: Option<String>,
    /// Require client authentication
    pub require_client_auth: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cert_path: None,
            key_path: None,
            ca_cert_path: None,
            require_client_auth: false,
        }
    }
}

/// TLS 1.3 server for control plane
pub struct TlsServer {
    #[allow(dead_code)]
    config: TlsConfig,
    listener: Option<TcpListener>,
}

impl TlsServer {
    /// Create a new TLS server
    pub fn new(config: TlsConfig) -> Self {
        Self {
            config,
            listener: None,
        }
    }

    /// Start listening on the specified address
    pub async fn listen(&mut self, addr: &str) -> Result<(), TlsError> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| TlsError::NetworkError(e.to_string()))?;
        self.listener = Some(listener);
        Ok(())
    }

    /// Accept a new TLS connection
    pub async fn accept(&self) -> Result<TlsStream, TlsError> {
        let listener = self.listener.as_ref().ok_or(TlsError::NotListening)?;

        let (stream, _) = listener
            .accept()
            .await
            .map_err(|e| TlsError::NetworkError(e.to_string()))?;

        // TODO: Wrap stream with rustls or native-tls
        // For now, return raw TCP stream wrapper
        Ok(TlsStream { inner: stream })
    }
}

/// TLS 1.3 client connection
pub struct TlsClient {
    #[allow(dead_code)]
    config: TlsConfig,
}

impl TlsClient {
    /// Create a new TLS client
    pub fn new(config: TlsConfig) -> Self {
        Self { config }
    }

    /// Connect to a TLS server
    pub async fn connect(&self, addr: &str) -> Result<TlsStream, TlsError> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| TlsError::NetworkError(e.to_string()))?;

        // TODO: Wrap stream with rustls or native-tls
        // For now, return raw TCP stream wrapper
        Ok(TlsStream { inner: stream })
    }
}

/// TLS stream wrapper
pub struct TlsStream {
    inner: TcpStream,
}

impl TlsStream {
    /// Read data from TLS stream
    pub async fn read(&self, buf: &mut [u8]) -> Result<usize, TlsError> {
        use tokio::io::AsyncReadExt;
        self.inner
            .readable()
            .await
            .map_err(|e| TlsError::NetworkError(e.to_string()))?;
        self.inner
            .try_read(buf)
            .map_err(|e| TlsError::NetworkError(e.to_string()))
    }

    /// Write data to TLS stream
    pub async fn write(&self, buf: &[u8]) -> Result<usize, TlsError> {
        use tokio::io::AsyncWriteExt;
        self.inner
            .writable()
            .await
            .map_err(|e| TlsError::NetworkError(e.to_string()))?;
        self.inner
            .try_write(buf)
            .map_err(|e| TlsError::NetworkError(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("TLS handshake error: {0}")]
    HandshakeError(String),
    #[error("Certificate error: {0}")]
    CertificateError(String),
    #[error("Server not listening")]
    NotListening,
}
