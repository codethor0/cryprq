// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! TUN interface for system-wide VPN routing
//!
//! This module provides TUN interface creation and packet forwarding
//! to enable routing all system traffic through the encrypted tunnel.

use anyhow::{Context, Result};
use std::process::Command;
use std::sync::Arc;
use std::io::{Read, Write};

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
        let addr: std::net::Ipv4Addr = config.address.parse()
            .context("Invalid TUN address (must be IPv4)")?;
        let netmask: std::net::Ipv4Addr = config.netmask.parse()
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
        let addr: std::net::Ipv4Addr = config.address.parse()
            .context("Invalid TUN address (must be IPv4)")?;
        let netmask: std::net::Ipv4Addr = config.netmask.parse()
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
    pub async fn start_forwarding(&mut self, tunnel: Arc<crate::Tunnel>) -> Result<()> {
        log::info!("Starting packet forwarding for TUN interface {}", self.interface_name);
        
        let device = self.device.take()
            .context("TUN device not initialized")?;
        
        // Keep device alive by moving it into the tasks
        // We'll use blocking I/O wrapped in spawn_blocking
        // Note: tun::Device implements Read/Write, so we can use it directly
        let tunnel_read = tunnel.clone();
        
        // Spawn blocking task to read from TUN and send to tunnel
        // We need to share device for read/write, so we'll use Arc<Mutex<>> wrapper
        let device_clone = Arc::new(std::sync::Mutex::new(device));
        let device_read = device_clone.clone();
        let device_write = device_clone.clone();
        
        let tun_read_task = tokio::spawn(async move {
            loop {
                // Use blocking read in spawn_blocking
                let mut buf = vec![0u8; 65535];
                let n = match tokio::task::spawn_blocking({
                    let dev = device_read.clone();
                    let mut buf_clone = buf.clone();
                    move || {
                        let mut dev_guard = dev.lock().unwrap();
                        dev_guard.read(&mut buf_clone)
                    }
                }).await {
                    Ok(Ok(n)) => {
                        buf = vec![0u8; 65535]; // Reset buffer
                        n
                    },
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
                    log::debug!("Read {} bytes from TUN interface", n);
                    
                    // Encrypt and send via tunnel
                    if let Err(e) = tunnel_read.send_packet(&packet).await {
                        log::error!("Failed to send packet via tunnel: {}", e);
                    }
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
        });
        
        // Spawn task to receive from tunnel and write to TUN
        let tun_write_task = tokio::spawn(async move {
            loop {
                match tunnel.recv_packet().await {
                    Ok(packet) => {
                        log::debug!("Received {} bytes from tunnel, writing to TUN", packet.len());
                        // Use blocking write in spawn_blocking
                        let dev = device_write.clone();
                        if let Err(e) = tokio::task::spawn_blocking(move || {
                            let mut dev_guard = dev.lock().unwrap();
                            dev_guard.write_all(&packet)
                        }).await {
                            log::error!("Failed to write packet to TUN: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("Error receiving packet from tunnel: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });
        
        log::info!("Packet forwarding loop started - routing system traffic through encrypted tunnel");
        
        // Wait for tasks (they run forever)
        tokio::select! {
            _ = tun_read_task => {},
            _ = tun_write_task => {},
        }
        
        Ok(())
    }
}

