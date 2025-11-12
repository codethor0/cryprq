// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! TUN interface for system-wide VPN routing
//!
//! This module provides TUN interface creation and packet forwarding
//! to enable routing all system traffic through the encrypted tunnel.

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::io::{Read, Write};
use std::process::Command;
use std::sync::Arc;

/// TUN interface configuration
#[derive(Clone)]
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

/// Trait for packet forwarding - allows forwarding to different backends
#[async_trait]
pub trait PacketForwarder: Send + Sync {
    async fn send_packet(&self, packet: &[u8]) -> Result<()>;
    async fn recv_packet(&mut self) -> Result<Vec<u8>>;
}

/// TUN interface handle
pub struct TunInterface {
    config: TunConfig,
    interface_name: String,
    #[cfg(target_os = "macos")]
    device: Option<tun::platform::macos::Device>,
    #[cfg(target_os = "linux")]
    device: Option<tun::platform::linux::Device>,
}

impl TunInterface {
    /// Create a new TUN interface
    ///
    /// This creates and configures the TUN interface.
    /// Requires root/admin privileges on macOS/Linux.
    pub async fn create(config: TunConfig) -> Result<Self> {
        let interface_name = config.name.clone();

        // Create the TUN device using the tun crate (blocking, but fast)
        let device = tokio::task::spawn_blocking({
            let config_clone = config.clone();
            let name_clone = interface_name.clone();
            move || Self::create_device(&config_clone, &name_clone)
        })
        .await
        .context("Failed to spawn TUN creation task")?
        .context("Failed to create TUN device")?;

        Ok(Self {
            config,
            interface_name,
            device: Some(device),
        })
    }

    #[cfg(target_os = "macos")]
    fn create_device(config: &TunConfig, name: &str) -> Result<tun::platform::macos::Device> {
        log::info!("Creating TUN interface {} for VPN mode", name);

        let mut config_builder = tun::Configuration::default();
        let addr: std::net::Ipv4Addr = config
            .address
            .parse()
            .context("Invalid TUN address (must be IPv4)")?;
        let netmask: std::net::Ipv4Addr = config
            .netmask
            .parse()
            .context("Invalid netmask (must be IPv4)")?;

        config_builder
            .name(name)
            .address(addr)
            .netmask(netmask)
            .mtu(config.mtu as i32)
            .up();

        let device = tun::platform::macos::create(&config_builder)
            .context("Failed to create TUN device (requires root/admin privileges)")?;

        log::info!("TUN interface {} created successfully", name);
        Ok(device)
    }

    #[cfg(target_os = "linux")]
    fn create_device(config: &TunConfig, name: &str) -> Result<tun::platform::linux::Device> {
        log::info!("Creating TUN interface {} for VPN mode", name);

        let mut config_builder = tun::Configuration::default();
        let addr: std::net::Ipv4Addr = config
            .address
            .parse()
            .context("Invalid TUN address (must be IPv4)")?;
        let netmask: std::net::Ipv4Addr = config
            .netmask
            .parse()
            .context("Invalid netmask (must be IPv4)")?;

        config_builder
            .name(name)
            .address(addr)
            .netmask(netmask)
            .mtu(config.mtu as i32)
            .up();

        let device = tun::platform::linux::create(&config_builder)
            .context("Failed to create TUN device (requires root/admin privileges)")?;

        log::info!("TUN interface {} created successfully", name);
        Ok(device)
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
            .args([
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
        let addr = &self.config.address;
        let netmask = &self.config.netmask;

        // Use ip command to configure the interface
        // This requires root/admin privileges
        let output = Command::new("sudo")
            .args([
                "ip",
                "addr",
                "add",
                &format!("{}/{}", addr, self.netmask_to_cidr(netmask)?),
                "dev",
                name,
            ])
            .output()
            .context("Failed to configure TUN interface (ip addr)")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ip addr failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output = Command::new("sudo")
            .args(["ip", "link", "set", name, "up"])
            .output()
            .context("Failed to bring TUN interface up (ip link)")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ip link failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        log::info!("Configured TUN interface {} with IP {}", name, addr);
        Ok(())
    }

    fn netmask_to_cidr(&self, netmask: &str) -> Result<u8> {
        let parts: Vec<u8> = netmask
            .split('.')
            .map(|s| s.parse().context("Invalid netmask format"))
            .collect::<Result<Vec<u8>>>()?;

        if parts.len() != 4 {
            return Err(anyhow::anyhow!("Invalid netmask format"));
        }

        let mut cidr = 0;
        for part in parts {
            cidr += part.count_ones() as u8;
        }
        Ok(cidr)
    }

    /// Start packet forwarding loop with a generic PacketForwarder
    ///
    /// This forwards packets between the TUN interface and the packet forwarder.
    pub async fn start_forwarding<F: PacketForwarder + 'static>(
        &mut self,
        forwarder: Arc<tokio::sync::Mutex<F>>,
    ) -> Result<()> {
        log::info!(
            "Starting packet forwarding for TUN interface {}",
            self.interface_name
        );

        let device = self.device.take().context("TUN device not initialized")?;

        // Keep device alive by moving it into the tasks
        let device_clone = Arc::new(std::sync::Mutex::new(device));
        let device_read = device_clone.clone();
        let device_write = device_clone.clone();
        let forwarder_read = forwarder.clone();
        let forwarder_write = forwarder.clone();

        // Spawn task to read from TUN and send via forwarder
        let tun_read_task = tokio::spawn(async move {
            let buf = vec![0u8; 65535];
            loop {
                let n = match tokio::task::spawn_blocking({
                    let dev = device_read.clone();
                    let mut buf_clone = buf.clone();
                    move || {
                        let mut dev_guard = dev.lock().map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Mutex lock failed: {}", e),
                            )
                        })?;
                        dev_guard.read(&mut buf_clone)
                    }
                })
                .await
                {
                    Ok(Ok(n)) => n,
                    Ok(Err(e)) => {
                        log::error!("Error reading from TUN: {}", e);
                        break;
                    }
                    Err(e) => {
                        log::error!("Task error: {}", e);
                        break;
                    }
                };

                if n > 0 {
                    let packet = buf[..n].to_vec();
                    log::debug!("🔐 Read {} bytes from TUN, encrypting and forwarding", n);

                    // Send via forwarder
                    let fwd = forwarder_read.lock().await;
                    if let Err(e) = fwd.send_packet(&packet).await {
                        log::error!("Failed to forward packet: {}", e);
                    }
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
        });

        // Spawn task to receive from forwarder and write to TUN
        let tun_write_task = tokio::spawn(async move {
            loop {
                let packet = {
                    let mut fwd = forwarder_write.lock().await;
                    match fwd.recv_packet().await {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("Error receiving packet from forwarder: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            continue;
                        }
                    }
                };

                log::debug!(
                    "🔓 Received {} bytes from tunnel, decrypting and writing to TUN",
                    packet.len()
                );

                // Write to TUN
                if let Err(e) = tokio::task::spawn_blocking({
                    let dev = device_write.clone();
                    let pkt = packet.clone();
                    move || {
                        let mut dev_guard = dev.lock().map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Mutex lock failed: {}", e),
                            )
                        })?;
                        dev_guard.write_all(&pkt)
                    }
                })
                .await
                {
                    log::error!("Failed to write packet to TUN: {}", e);
                }
            }
        });

        log::info!(
            "✅ Packet forwarding loop started - routing system traffic through encrypted tunnel"
        );

        // Wait for tasks (they run forever)
        tokio::select! {
            _ = tun_read_task => {},
            _ = tun_write_task => {},
        }

        Ok(())
    }
}
