// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

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

/// Smallest MTU an IPv4 link may have (RFC 791).
const MIN_MTU: u16 = 576;

impl TunConfig {
    /// Checks the address, netmask and MTU before any device is requested and
    /// returns the parsed address and netmask.
    ///
    /// The device gets a connected route for its subnet. A subnet wider than
    /// /8, or an address that already means something else on every host
    /// (loopback, link-local, multicast, broadcast, unspecified), would
    /// capture traffic that was never meant for the tunnel, so those are
    /// refused here instead of being handed to the system.
    pub fn validate(&self) -> Result<(std::net::Ipv4Addr, std::net::Ipv4Addr)> {
        let address: std::net::Ipv4Addr = self
            .address
            .parse()
            .context("Invalid TUN address (must be IPv4)")?;
        let netmask: std::net::Ipv4Addr = self
            .netmask
            .parse()
            .context("Invalid netmask (must be IPv4)")?;

        if address.is_unspecified()
            || address.is_loopback()
            || address.is_link_local()
            || address.is_multicast()
            || address.is_broadcast()
        {
            anyhow::bail!("TUN address {address} is reserved and cannot be assigned to a tunnel");
        }
        let mask = u32::from(netmask);
        if mask.leading_ones() + mask.trailing_zeros() != 32 {
            anyhow::bail!("Netmask {netmask} is not a contiguous prefix");
        }
        let prefix = mask.leading_ones();
        if !(8..=30).contains(&prefix) {
            anyhow::bail!("Netmask {netmask} (/{prefix}) is outside the supported range /8 to /30");
        }
        let host = u32::from(address) & !mask;
        if host == 0 || host == !mask {
            anyhow::bail!(
                "TUN address {address} is the network or broadcast address of its /{prefix} subnet"
            );
        }
        if self.mtu < MIN_MTU {
            anyhow::bail!(
                "TUN MTU {} is below the IPv4 minimum of {MIN_MTU}",
                self.mtu
            );
        }
        Ok((address, netmask))
    }
}

/// Largest IP packet a tunnel carries.
pub const MAX_IP_PACKET_LEN: usize = 65535;

/// Bytes of packet information the platform's TUN device puts in front of
/// every packet: a 4-byte address family on macOS, nothing on Linux (the
/// device is opened with `IFF_NO_PI`).
#[cfg(target_os = "macos")]
const PACKET_INFO_LEN: usize = 4;
#[cfg(not(target_os = "macos"))]
const PACKET_INFO_LEN: usize = 0;

/// Address families of the macOS utun packet information header.
const UTUN_AF_INET: u8 = 2;
const UTUN_AF_INET6: u8 = 30;

/// IP version (4 or 6) of `packet`, or `None` if it is neither.
pub fn ip_version(packet: &[u8]) -> Option<u8> {
    match packet.first().map(|first| first >> 4) {
        Some(4) if packet.len() >= 20 => Some(4),
        Some(6) if packet.len() >= 40 => Some(6),
        _ => None,
    }
}

/// The device name to ask macOS for. TUN devices there are always called
/// `utun<N>`; for any other name the system picks a free one.
#[cfg(any(target_os = "macos", test))]
fn utun_request(name: &str) -> Option<&str> {
    let number = name.strip_prefix("utun")?;
    let valid =
        !number.is_empty() && number.len() <= 4 && number.bytes().all(|b| b.is_ascii_digit());
    valid.then_some(name)
}

/// The IP packet inside a frame read from the device.
fn strip_packet_info(frame: &[u8], info_len: usize) -> Option<&[u8]> {
    let packet = frame.get(info_len..)?;
    ip_version(packet).map(|_| packet)
}

/// The frame to write to the device for `packet`; `None` for anything that is
/// not an IPv4 or IPv6 packet.
fn add_packet_info(packet: &[u8], info_len: usize) -> Option<Vec<u8>> {
    let version = ip_version(packet)?;
    if packet.len() > MAX_IP_PACKET_LEN {
        return None;
    }
    let mut frame = Vec::with_capacity(info_len + packet.len());
    if info_len == 4 {
        let family = if version == 4 {
            UTUN_AF_INET
        } else {
            UTUN_AF_INET6
        };
        frame.extend_from_slice(&[0, 0, 0, family]);
    }
    frame.extend_from_slice(packet);
    Some(frame)
}

/// `(packets read from the device, packets to write to the device)`.
pub type PacketChannels = (
    tokio::sync::mpsc::Receiver<Vec<u8>>,
    tokio::sync::mpsc::Sender<Vec<u8>>,
);

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
        config.validate()?;

        // Create the TUN device using the tun crate (blocking, but fast)
        let device = tokio::task::spawn_blocking({
            let config_clone = config.clone();
            let name_clone = interface_name.clone();
            move || Self::create_device(&config_clone, &name_clone)
        })
        .await
        .context("Failed to spawn TUN creation task")?
        .context("Failed to create TUN device")?;

        // The system may have chosen the name (see `utun_request`).
        #[cfg(target_os = "macos")]
        let interface_name = {
            use tun::Device as _;
            device.name().unwrap_or(interface_name)
        };

        Ok(Self {
            config,
            interface_name,
            device: Some(device),
        })
    }

    #[cfg(target_os = "macos")]
    fn create_device(config: &TunConfig, name: &str) -> Result<tun::platform::macos::Device> {
        use tun::Device as _;

        let requested = utun_request(name);
        match requested {
            Some(name) => log::info!("Creating TUN interface {name} for VPN mode"),
            None => log::info!(
                "Creating a TUN interface for VPN mode; macOS names it utun<N>, not {name}"
            ),
        }

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
            .address(addr)
            .netmask(netmask)
            .mtu(config.mtu as i32)
            .up();
        if let Some(name) = requested {
            config_builder.name(name);
        }

        let device = tun::platform::macos::create(&config_builder)
            .context("Failed to create TUN device (requires root/admin privileges)")?;

        log::info!(
            "TUN interface {} created successfully",
            device.name().unwrap_or_default()
        );
        Ok(device)
    }

    #[cfg(target_os = "linux")]
    fn create_device(config: &TunConfig, name: &str) -> Result<tun::platform::linux::Device> {
        log::info!("Creating TUN interface {name} for VPN mode");

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

        log::info!("TUN interface {name} created successfully");
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

        log::info!("Configured TUN interface {name} with IP {addr}");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn configure_ip_linux(&self) -> Result<()> {
        let name = self.name();
        let addr = &self.config.address;
        let netmask = &self.config.netmask;

        // The device was created with its address, so the address is
        // normally there already: `replace` sets it either way, where `add`
        // fails on an address that exists. The process that could create the
        // device has the privilege for this too, so there is no `sudo`.
        let output = Command::new("ip")
            .args([
                "addr",
                "replace",
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

        let output = Command::new("ip")
            .args(["link", "set", name, "up"])
            .output()
            .context("Failed to bring TUN interface up (ip link)")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "ip link failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        log::info!("Configured TUN interface {name} with IP {addr}");
        Ok(())
    }

    #[allow(dead_code)]
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

    /// Turns the device into a pair of packet channels and returns
    /// `(packets read from the device, packets to write to the device)`.
    ///
    /// Reading and writing run on two dedicated threads over the two halves of
    /// the device, so a blocked read never delays a write. Packets are whole IP
    /// packets without any platform framing. When a channel is full the packet
    /// is dropped, as a congested link would.
    pub fn into_channels(mut self, capacity: usize) -> Result<PacketChannels> {
        use tokio::sync::mpsc::{self, error::TrySendError};

        let device = self.device.take().context("TUN device not initialized")?;
        let (mut reader, mut writer) = device.split();
        let (from_device_tx, from_device_rx) = mpsc::channel::<Vec<u8>>(capacity);
        let (to_device_tx, mut to_device_rx) = mpsc::channel::<Vec<u8>>(capacity);
        let name = self.interface_name.clone();

        std::thread::Builder::new()
            .name(format!("{name}-read"))
            .spawn(move || {
                let mut buf = vec![0u8; MAX_IP_PACKET_LEN + PACKET_INFO_LEN];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let Some(packet) = strip_packet_info(&buf[..n], PACKET_INFO_LEN) else {
                                continue;
                            };
                            if let Err(TrySendError::Closed(_)) =
                                from_device_tx.try_send(packet.to_vec())
                            {
                                break;
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                        Err(e) => {
                            log::error!("Error reading from TUN: {e}");
                            break;
                        }
                    }
                }
            })
            .context("Failed to start the TUN read thread")?;

        std::thread::Builder::new()
            .name(format!("{}-write", self.interface_name))
            .spawn(move || {
                while let Some(packet) = to_device_rx.blocking_recv() {
                    let Some(frame) = add_packet_info(&packet, PACKET_INFO_LEN) else {
                        log::debug!("Dropping a tunnelled packet that is not IPv4 or IPv6");
                        continue;
                    };
                    if let Err(e) = writer.write_all(&frame) {
                        log::warn!("Failed to write packet to TUN: {e}");
                    }
                }
            })
            .context("Failed to start the TUN write thread")?;

        Ok((from_device_rx, to_device_tx))
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
        let mut tun_read_task = tokio::spawn(async move {
            let buf = vec![0u8; 65535];
            loop {
                let n = match tokio::task::spawn_blocking({
                    let dev = device_read.clone();
                    let mut buf_clone = buf.clone();
                    move || {
                        let mut dev_guard = dev.lock().map_err(|e| {
                            std::io::Error::other(format!("Mutex lock failed: {e}"))
                        })?;
                        dev_guard.read(&mut buf_clone)
                    }
                })
                .await
                {
                    Ok(Ok(n)) => n,
                    Ok(Err(e)) => {
                        log::error!("Error reading from TUN: {e}");
                        break;
                    }
                    Err(e) => {
                        log::error!("Task error: {e}");
                        break;
                    }
                };

                if n > 0 {
                    let packet = buf[..n].to_vec();
                    log::debug!("Read {n} bytes from TUN, encrypting and forwarding");

                    // Send via forwarder
                    let fwd = forwarder_read.lock().await;
                    if let Err(e) = fwd.send_packet(&packet).await {
                        log::error!("Failed to forward packet: {e}");
                    }
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
        });

        // Spawn task to receive from forwarder and write to TUN
        let mut tun_write_task = tokio::spawn(async move {
            loop {
                let packet = {
                    let mut fwd = forwarder_write.lock().await;
                    match fwd.recv_packet().await {
                        Ok(p) => p,
                        Err(e) => {
                            log::error!("Error receiving packet from forwarder: {e}");
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            continue;
                        }
                    }
                };

                log::debug!(
                    "Received {} bytes from tunnel, decrypting and writing to TUN",
                    packet.len()
                );

                // Write to TUN
                if let Err(e) = tokio::task::spawn_blocking({
                    let dev = device_write.clone();
                    let pkt = packet.clone();
                    move || {
                        let mut dev_guard = dev.lock().map_err(|e| {
                            std::io::Error::other(format!("Mutex lock failed: {e}"))
                        })?;
                        dev_guard.write_all(&pkt)
                    }
                })
                .await
                {
                    log::error!("Failed to write packet to TUN: {e}");
                }
            }
        });

        log::info!(
            "Packet forwarding loop started - routing system traffic through encrypted tunnel"
        );

        // Wait for tasks (they run forever)
        std::future::poll_fn(|cx| {
            use std::{future::Future, pin::Pin, task::Poll};
            if Pin::new(&mut tun_read_task).poll(cx).is_ready()
                || Pin::new(&mut tun_write_task).poll(cx).is_ready()
            {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ipv4(len: usize) -> Vec<u8> {
        let mut packet = vec![0u8; len];
        packet[0] = 0x45;
        packet
    }

    #[test]
    fn ip_version_requires_a_complete_header() {
        assert_eq!(ip_version(&ipv4(20)), Some(4));
        assert_eq!(ip_version(&ipv4(19)), None);
        let mut v6 = vec![0u8; 40];
        v6[0] = 0x60;
        assert_eq!(ip_version(&v6), Some(6));
        assert_eq!(ip_version(&v6[..39]), None);
        assert_eq!(ip_version(&[]), None);
        assert_eq!(ip_version(&[0x55; 64]), None);
    }

    #[test]
    fn packet_information_round_trips_per_platform() {
        let v4 = ipv4(28);
        let mut v6 = vec![0u8; 48];
        v6[0] = 0x60;

        // Linux: no framing at all.
        assert_eq!(add_packet_info(&v4, 0).as_deref(), Some(v4.as_slice()));
        assert_eq!(strip_packet_info(&v4, 0), Some(v4.as_slice()));

        // macOS: a 4-byte address family in network byte order.
        let framed_v4 = add_packet_info(&v4, 4).expect("ipv4 frame");
        assert_eq!(framed_v4[..4], [0, 0, 0, 2]);
        assert_eq!(strip_packet_info(&framed_v4, 4), Some(v4.as_slice()));
        let framed_v6 = add_packet_info(&v6, 4).expect("ipv6 frame");
        assert_eq!(framed_v6[..4], [0, 0, 0, 30]);
        assert_eq!(strip_packet_info(&framed_v6, 4), Some(v6.as_slice()));
    }

    fn config(address: &str, netmask: &str, mtu: u16) -> TunConfig {
        TunConfig {
            name: "utun9".to_string(),
            address: address.to_string(),
            netmask: netmask.to_string(),
            mtu,
        }
    }

    #[test]
    fn usable_tunnel_addresses_are_accepted() {
        assert!(TunConfig::default().validate().is_ok());
        for (address, netmask) in [
            ("10.0.0.1", "255.255.255.0"),
            ("10.77.0.2", "255.255.255.252"),
            ("172.16.5.9", "255.255.0.0"),
            ("100.64.0.1", "255.0.0.0"),
        ] {
            let parsed = config(address, netmask, 1420).validate();
            assert!(parsed.is_ok(), "{address}/{netmask}: {parsed:?}");
        }
    }

    #[test]
    fn addresses_that_would_capture_other_traffic_are_refused() {
        for (address, netmask, mtu) in [
            // A subnet this wide would act as a default route.
            ("10.0.0.1", "0.0.0.0", 1420),
            ("10.0.0.1", "128.0.0.0", 1420),
            ("10.0.0.1", "254.0.0.0", 1420),
            // Not a prefix.
            ("10.0.0.1", "255.0.255.0", 1420),
            ("10.0.0.1", "0.255.255.255", 1420),
            // No room for two hosts.
            ("10.0.0.1", "255.255.255.254", 1420),
            ("10.0.0.1", "255.255.255.255", 1420),
            // Reserved addresses.
            ("0.0.0.0", "255.255.255.0", 1420),
            ("127.0.0.1", "255.255.255.0", 1420),
            ("169.254.1.1", "255.255.0.0", 1420),
            ("224.0.0.1", "255.255.255.0", 1420),
            ("255.255.255.255", "255.255.255.0", 1420),
            // Network and broadcast address of the subnet.
            ("10.0.0.0", "255.255.255.0", 1420),
            ("10.0.0.255", "255.255.255.0", 1420),
            // Not IPv4 at all.
            ("fd00::1", "255.255.255.0", 1420),
            ("10.0.0", "255.255.255.0", 1420),
            ("10.0.0.1", "24", 1420),
            ("", "", 1420),
            // An MTU no IPv4 link may have.
            ("10.0.0.1", "255.255.255.0", 575),
            ("10.0.0.1", "255.255.255.0", 0),
        ] {
            assert!(
                config(address, netmask, mtu).validate().is_err(),
                "{address}/{netmask} mtu {mtu} was accepted"
            );
        }
    }

    #[test]
    fn only_utun_names_are_requested_from_macos() {
        assert_eq!(utun_request("utun7"), Some("utun7"));
        assert_eq!(utun_request("utun0"), Some("utun0"));
        for name in [
            "cryprq0",
            "utun",
            "utun-1",
            "utunx",
            "utun12345",
            "tun0",
            "",
        ] {
            assert_eq!(utun_request(name), None, "{name}");
        }
    }

    #[test]
    fn frames_that_are_not_ip_never_reach_or_leave_the_device() {
        assert_eq!(add_packet_info(b"not an ip packet at all......", 0), None);
        assert_eq!(add_packet_info(&[], 4), None);
        assert_eq!(strip_packet_info(&[0, 0, 0, 2], 4), None);
        assert_eq!(strip_packet_info(&[0, 0], 4), None);
        assert_eq!(add_packet_info(&ipv4(MAX_IP_PACKET_LEN + 1), 0), None);
    }
}
