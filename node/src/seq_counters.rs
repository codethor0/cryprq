// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Sequence counters for CrypRQ record layer
//!
//! Per-stream or per-message-type sequence number management

use std::sync::atomic::{AtomicU64, Ordering};

/// Sequence counters for different message types
#[derive(Debug)]
pub struct SeqCounters {
    vpn: AtomicU64,
    data: AtomicU64,
    file: AtomicU64,
}

impl SeqCounters {
    /// Create new sequence counters (all start at 0)
    pub fn new() -> Self {
        Self {
            vpn: AtomicU64::new(0),
            data: AtomicU64::new(0),
            file: AtomicU64::new(0),
        }
    }

    /// Get next sequence number for VPN packets
    pub fn next_vpn(&self) -> u64 {
        self.vpn.fetch_add(1, Ordering::Relaxed)
    }

    /// Get next sequence number for generic data
    pub fn next_data(&self) -> u64 {
        self.data.fetch_add(1, Ordering::Relaxed)
    }

    /// Get next sequence number for file transfer
    pub fn next_file(&self) -> u64 {
        self.file.fetch_add(1, Ordering::Relaxed)
    }

    /// Reset all counters (used on epoch change)
    pub fn reset(&self) {
        self.vpn.store(0, Ordering::Relaxed);
        self.data.store(0, Ordering::Relaxed);
        self.file.store(0, Ordering::Relaxed);
    }
}

impl Default for SeqCounters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_counters() {
        let counters = SeqCounters::new();
        assert_eq!(counters.next_vpn(), 0);
        assert_eq!(counters.next_vpn(), 1);
        assert_eq!(counters.next_file(), 0);
        assert_eq!(counters.next_data(), 0);
    }

    #[test]
    fn test_reset() {
        let counters = SeqCounters::new();
        counters.next_vpn();
        counters.next_file();
        counters.reset();
        assert_eq!(counters.next_vpn(), 0);
        assert_eq!(counters.next_file(), 0);
    }
}

