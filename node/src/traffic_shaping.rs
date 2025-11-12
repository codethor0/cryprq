// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Traffic Shaping for Analysis Resistance
//!
//! This module implements constant-rate traffic generation and
//! packet timing obfuscation to resist traffic analysis attacks.

use std::time::{Duration, Instant};
use tokio::time::{interval, MissedTickBehavior};

/// Traffic shaper for constant-rate traffic generation
pub struct TrafficShaper {
    /// Target packets per second
    pps: f64,
    /// Last packet timestamp
    last_packet: Option<Instant>,
    /// Interval timer
    interval: tokio::time::Interval,
}

impl TrafficShaper {
    /// Create a new traffic shaper
    ///
    /// # Arguments
    ///
    /// * `packets_per_second` - Target packet rate
    pub fn new(packets_per_second: f64) -> Self {
        let interval_ms = (1000.0 / packets_per_second) as u64;
        let mut interval = interval(Duration::from_millis(interval_ms));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        Self {
            pps: packets_per_second,
            last_packet: None,
            interval,
        }
    }

    /// Wait for the next packet slot (for constant-rate traffic)
    pub async fn wait_for_slot(&mut self) {
        self.interval.tick().await;
        self.last_packet = Some(Instant::now());
    }

    /// Check if it's time to send a packet (for constant-rate traffic)
    pub fn should_send(&mut self) -> bool {
        if let Some(last) = self.last_packet {
            let elapsed = last.elapsed();
            let target_interval = Duration::from_secs_f64(1.0 / self.pps);
            elapsed >= target_interval
        } else {
            true
        }
    }

    /// Add jitter to packet timing to avoid perfect constant-rate detection
    ///
    /// Returns a random delay between 0 and `max_jitter_ms` milliseconds
    pub fn jitter_delay(&self, max_jitter_ms: u64) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter_ms = rng.gen_range(0..=max_jitter_ms);
        Duration::from_millis(jitter_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_traffic_shaper_creation() {
        let shaper = TrafficShaper::new(10.0);
        assert_eq!(shaper.pps, 10.0);
    }

    #[tokio::test]
    async fn test_should_send() {
        let mut shaper = TrafficShaper::new(1.0);
        assert!(shaper.should_send()); // Should send immediately if no previous packet
    }
}
