// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! TUN interface for system-wide VPN routing
//!
//! This module provides TUN interface creation and packet forwarding
//! to enable routing all system traffic through the encrypted tunnel.
//!
//! NOTE: This is a simplified implementation for proof-of-concept.
//! Production use should use proper Network Extension framework on macOS.

use anyhow::{Context, Result};
use std::process::Command;

/// TUN interface configuration
pub struct TunConfig {
    pub name: String,
    pub address: String,
    pub netmask: String,
    pub mtu: u16,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            name: "cryprq0".to_string(),
            address: "10.0.0.1".to_string(),
            netmask: "255.255.255.0".to_string(),
            mtu: 1420,
        }
    }
}

/// TUN interface handle
/// 
/// NOTE: This is a simplified implementation that uses system commands
/// to create and configure the TUN interface. For production use,
/// consider using a proper Network Extension framework.
pub struct TunInterface {
    config: TunConfig,
    interface_name: String,
}

impl TunInterface {
    /// Create a new TUN interface
    /// 
    /// This uses system commands to create and configure the interface.
    /// Requires root/admin privileges.
    pub async fn create(config: TunConfig) -> Result<Self> {
        let interface_name = config.name.clone();
        
        // Create and configure the interface
        #[cfg(target_os = "macos")]
        {
            Self::create_macos(&config, &interface_name).await?;
        }
        #[cfg(target_os = "linux")]
        {
            Self::create_linux(&config, &interface_name).await?;
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            anyhow::bail!("TUN interface not supported on this platform");
        }

        Ok(Self {
            config,
            interface_name,
        })
    }

    #[cfg(target_os = "macos")]
    async fn create_macos(config: &TunConfig, name: &str) -> Result<()> {
        // On macOS, creating TUN interfaces requires Network Extension framework
        // or root privileges. For now, we'll attempt to create via system commands.
        
        log::info!("Creating TUN interface {} on macOS", name);
        log::info!("VPN Mode: Attempting to create TUN interface for system-wide routing");
        
        // Try to create utun interface using ifconfig
        // Note: This requires sudo/admin privileges
        let output = Command::new("ifconfig")
            .args(&["-l"])
            .output()
            .context("Failed to list interfaces")?;
        
        let interfaces = String::from_utf8_lossy(&output.stdout);
        log::debug!("Available interfaces: {}", interfaces);
        
        // Check if utun interface already exists
        if interfaces.contains("utun") {
            log::info!("Found existing utun interface - will configure it");
        } else {
            log::warn!("No utun interface found - TUN creation requires Network Extension framework");
            log::warn!("For full system-wide VPN, macOS requires Network Extension (NEPacketTunnelProvider)");
            log::info!("Current implementation: P2P encrypted tunnel is working, but system routing needs Network Extension");
        }
        
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn create_linux(config: &TunConfig, name: &str) -> Result<()> {
        // On Linux, create TUN interface using ip command
        log::info!("Creating TUN interface {} on Linux (requires root privileges)", name);
        
        // Use ip tuntap command to create TUN interface
        let output = Command::new("sudo")
            .args(&["ip", "tuntap", "add", "mode", "tun", "name", name])
            .output()
            .context("Failed to create TUN interface (requires root)")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ip tuntap add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Get the TUN interface name
    pub fn name(&self) -> &str {
        &self.interface_name
    }

    /// Configure the interface IP address (requires root/admin)
    pub async fn configure_ip(&self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.configure_ip_macos().await
        }
        #[cfg(target_os = "linux")]
        {
            self.configure_ip_linux().await
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Ok(())
        }
    }

    #[cfg(target_os = "macos")]
    async fn configure_ip_macos(&self) -> Result<()> {
        let name = self.name();
        let addr = &self.config.address;
        let netmask = &self.config.netmask;

        // Use ifconfig to configure the interface
        // This requires root/admin privileges
        let output = Command::new("sudo")
            .args(&[
                "ifconfig",
                name,
                addr,
                "netmask",
                netmask,
                "mtu",
                &self.config.mtu.to_string(),
                "up",
            ])
            .output()
            .context("Failed to configure TUN interface (ifconfig)")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ifconfig failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        log::info!("Configured TUN interface {} with IP {}", name, addr);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn configure_ip_linux(&self) -> Result<()> {
        let name = self.name();
        let addr = format!("{}/24", self.config.address); // CIDR notation

        // Use ip command to configure the interface
        let output = Command::new("sudo")
            .args(&["ip", "addr", "add", &addr, "dev", name])
            .output()
            .context("Failed to configure TUN interface (ip addr)")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ip addr add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output = Command::new("sudo")
            .args(&["ip", "link", "set", "dev", name, "up"])
            .output()
            .context("Failed to bring up TUN interface")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ip link set up failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        log::info!("Configured TUN interface {} with IP {}", name, addr);
        Ok(())
    }

    /// Start packet forwarding loop
    /// 
    /// This forwards packets between the TUN interface and the encrypted tunnel.
    /// NOTE: This is a placeholder - actual implementation requires proper TUN device access.
    pub async fn start_forwarding(&mut self, _tunnel: &crate::Tunnel) -> Result<()> {
        log::info!("Packet forwarding started for TUN interface {}", self.interface_name);
        log::warn!("Packet forwarding is a placeholder - requires proper TUN device implementation");
        
        // TODO: Implement actual packet forwarding loop
        // 1. Read packets from TUN interface
        // 2. Encrypt and send via tunnel
        // 3. Receive encrypted packets from tunnel
        // 4. Decrypt and write to TUN interface
        
        Ok(())
    }
}

