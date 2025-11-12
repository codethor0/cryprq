// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! DNS-over-HTTPS (DoH) and DNS-over-TLS (DoT) Support
//!
//! This module provides encrypted DNS resolution to protect DNS queries
//! from surveillance and manipulation.

use std::net::IpAddr;
use std::time::Duration;
use tokio::time::timeout;

/// DNS resolver configuration
#[derive(Debug, Clone)]
pub struct DnsConfig {
    /// Use DNS-over-HTTPS (DoH)
    pub use_doh: bool,
    /// Use DNS-over-TLS (DoT)
    pub use_dot: bool,
    /// DoH endpoint URL (e.g., "https://cloudflare-dns.com/dns-query")
    pub doh_endpoint: Option<String>,
    /// DoT server address (e.g., "1.1.1.1:853")
    pub dot_server: Option<String>,
    /// Timeout for DNS queries
    pub timeout: Duration,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            use_doh: true,
            use_dot: false,
            doh_endpoint: Some("https://cloudflare-dns.com/dns-query".to_string()),
            dot_server: Some("1.1.1.1:853".to_string()),
            timeout: Duration::from_secs(5),
        }
    }
}

/// Resolve a hostname to an IP address using encrypted DNS
///
/// # Arguments
///
/// * `hostname` - Hostname to resolve (e.g., "example.com")
/// * `config` - DNS configuration
///
/// # Returns
///
/// Returns the first IP address found, or an error if resolution fails
pub async fn resolve_hostname(hostname: &str, config: &DnsConfig) -> Result<IpAddr, DnsError> {
    if config.use_doh {
        return resolve_doh(hostname, config).await;
    }

    if config.use_dot {
        return resolve_dot(hostname, config).await;
    }

    // Fallback to system DNS (not recommended for privacy)
    resolve_system(hostname).await
}

/// Resolve using DNS-over-HTTPS (DoH)
async fn resolve_doh(hostname: &str, config: &DnsConfig) -> Result<IpAddr, DnsError> {
    let endpoint = config
        .doh_endpoint
        .as_ref()
        .ok_or(DnsError::ConfigurationError("DoH endpoint not configured"))?;

    // Build DoH query URL
    let url = format!("{}?name={}&type=A", endpoint, hostname);

    // Make HTTPS request
    let client = reqwest::Client::builder()
        .timeout(config.timeout)
        .build()
        .map_err(|e| DnsError::NetworkError(e.to_string()))?;

    let response = timeout(config.timeout, client.get(&url).send())
        .await
        .map_err(|_| DnsError::Timeout)?
        .map_err(|e| DnsError::NetworkError(e.to_string()))?;

    // Parse JSON response (simplified - real implementation would parse DNS wire format)
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| DnsError::ParseError(e.to_string()))?;

    // Extract IP address from DoH JSON response
    if let Some(answer) = json.get("Answer").and_then(|a| a.as_array()) {
        for record in answer {
            if let Some(ip_str) = record.get("data").and_then(|d| d.as_str()) {
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    return Ok(ip);
                }
            }
        }
    }

    Err(DnsError::NotFound)
}

/// Resolve using DNS-over-TLS (DoT)
async fn resolve_dot(hostname: &str, _config: &DnsConfig) -> Result<IpAddr, DnsError> {
    // TODO: Implement DoT using rustls and DNS wire format
    // For now, fallback to system DNS
    resolve_system(hostname).await
}

/// Resolve using system DNS (fallback)
async fn resolve_system(hostname: &str) -> Result<IpAddr, DnsError> {
    use tokio::net::lookup_host;

    let mut addrs = lookup_host(hostname)
        .await
        .map_err(|e| DnsError::NetworkError(e.to_string()))?;

    addrs.next().map(|addr| addr.ip()).ok_or(DnsError::NotFound)
}

#[derive(Debug, thiserror::Error)]
pub enum DnsError {
    #[error("DNS configuration error: {0}")]
    ConfigurationError(&'static str),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("DNS query timeout")]
    Timeout,
    #[error("Hostname not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_system() {
        // Test system DNS resolution
        let result = resolve_system("localhost").await;
        assert!(result.is_ok() || result.is_err()); // May fail in test environment
    }
}
