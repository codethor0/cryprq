// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Traffic Analysis Resistance
//!
//! This module provides padding and constant-rate traffic generation
//! to resist traffic analysis attacks.

use rand::Rng;

/// Padding configuration for traffic analysis resistance
#[derive(Debug, Clone)]
pub struct PaddingConfig {
    /// Minimum packet size (bytes)
    pub min_packet_size: usize,
    /// Maximum packet size (bytes)
    pub max_packet_size: usize,
    /// Target packet size for constant-rate traffic
    pub target_packet_size: usize,
    /// Enable constant-rate traffic generation
    pub constant_rate: bool,
    /// Interval for constant-rate packets (milliseconds)
    pub constant_rate_interval_ms: u64,
}

impl Default for PaddingConfig {
    fn default() -> Self {
        Self {
            min_packet_size: 64,            // Minimum to avoid tiny packet detection
            max_packet_size: 1500,          // Ethernet MTU
            target_packet_size: 512,        // Target size for padding
            constant_rate: false,           // Disabled by default (performance)
            constant_rate_interval_ms: 100, // 10 packets/second
        }
    }
}

/// Add padding to a packet to resist traffic analysis
///
/// Pads packets to a target size using random padding to prevent
/// packet size fingerprinting attacks.
pub fn pad_packet(data: &[u8], config: &PaddingConfig) -> Vec<u8> {
    let current_size = data.len();

    // If already at or above target, return as-is (or pad to max if needed)
    if current_size >= config.target_packet_size {
        if current_size > config.max_packet_size {
            // Truncate if too large (shouldn't happen in practice)
            return data[..config.max_packet_size].to_vec();
        }
        return data.to_vec();
    }

    // Calculate padding needed
    let padding_needed = config.target_packet_size - current_size;

    // Generate random padding (not just zeros to avoid detection)
    let mut padded = Vec::with_capacity(config.target_packet_size);
    padded.extend_from_slice(data);

    // Add random padding bytes
    let mut rng = rand::thread_rng();
    for _ in 0..padding_needed {
        padded.push(rng.gen());
    }

    padded
}

/// Remove padding from a packet
///
/// In practice, padding removal depends on the protocol.
/// This is a simple implementation that assumes the original
/// data length is encoded in the packet header.
pub fn unpad_packet(padded: &[u8], original_len: usize) -> Vec<u8> {
    if original_len > padded.len() {
        return padded.to_vec(); // Return as-is if invalid
    }
    padded[..original_len].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_packet() {
        let config = PaddingConfig::default();
        let data = vec![1u8; 100];
        let padded = pad_packet(&data, &config);

        assert_eq!(padded.len(), config.target_packet_size);
        assert_eq!(&padded[..100], &data);
    }

    #[test]
    fn test_unpad_packet() {
        let padded = vec![1u8, 2u8, 3u8, 4u8, 5u8];
        let unpadded = unpad_packet(&padded, 3);

        assert_eq!(unpadded.len(), 3);
        assert_eq!(unpadded, vec![1u8, 2u8, 3u8]);
    }
}
