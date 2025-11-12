// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Packet forwarding over libp2p request-response protocol
//!
//! This module provides packet forwarding using libp2p's request-response protocol
//! for bidirectional packet exchange over encrypted streams.

use anyhow::Result;
use futures::{AsyncRead, AsyncWrite};
use libp2p::{request_response::Codec, swarm::Swarm, PeerId, StreamProtocol};
use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::MyBehaviour;

/// Simple codec for packet forwarding
#[derive(Clone, Default)]
pub struct PacketCodec;

/// Packet protocol identifier
pub const PACKET_PROTOCOL: StreamProtocol = StreamProtocol::new("/cryprq/packet/1.0.0");

#[async_trait::async_trait]
impl Codec for PacketCodec {
    type Protocol = String;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        if len > 65535 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Packet too large",
            ));
        }
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        Ok(buf)
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;
        let mut len_bytes = [0u8; 4];
        io.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        if len > 65535 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Packet too large",
            ));
        }
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        Ok(buf)
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;
        let len = req.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;
        io.write_all(&req).await?;
        io.flush().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;
        let len = res.len() as u32;
        io.write_all(&len.to_be_bytes()).await?;
        io.write_all(&res).await?;
        io.flush().await?;
        Ok(())
    }
}

/// Packet forwarder using libp2p request-response protocol
pub struct Libp2pPacketForwarder {
    #[allow(dead_code)]
    swarm: Arc<tokio::sync::Mutex<Swarm<MyBehaviour>>>,
    #[allow(dead_code)]
    peer_id: PeerId,
    send_tx: Arc<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
    recv_rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>>,
    recv_tx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>>,
}

// Type alias for new() return tuple to reduce complexity
type NewReturn = (
    Libp2pPacketForwarder,
    Arc<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
    Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>>,
);

impl Libp2pPacketForwarder {
    #[allow(clippy::type_complexity)]
    pub fn new(swarm: Arc<tokio::sync::Mutex<Swarm<MyBehaviour>>>, peer_id: PeerId) -> NewReturn {
        let (send_tx, mut send_rx) = tokio::sync::mpsc::unbounded_channel();
        let (recv_tx, recv_rx) = tokio::sync::mpsc::unbounded_channel();

        let send_tx_arc = Arc::new(send_tx.clone());
        let recv_rx_arc = Arc::new(Mutex::new(recv_rx));
        let recv_tx_arc = Arc::new(tokio::sync::Mutex::new(recv_tx));

        let forwarder = Self {
            swarm: swarm.clone(),
            peer_id,
            send_tx: send_tx_arc.clone(),
            recv_rx: recv_rx_arc.clone(),
            recv_tx: recv_tx_arc.clone(),
        };

        // Spawn task to handle sending packets via libp2p request-response
        let swarm_clone = swarm.clone();
        let peer_id_clone = peer_id;
        tokio::spawn(async move {
            loop {
                if let Some(packet) = send_rx.recv().await {
                    // Send packet as request via request-response protocol
                    let mut swarm_guard = swarm_clone.lock().await;

                    // Send request (packet) to peer using request-response behaviour
                    let request_id = swarm_guard
                        .behaviour_mut()
                        .request_response
                        .send_request(&peer_id_clone, packet.clone());
                    log::debug!(
                        "🔐 ENCRYPT: Sent {} bytes packet to {} (request_id: {:?})",
                        packet.len(),
                        peer_id_clone,
                        request_id
                    );
                }
            }
        });

        (forwarder, send_tx_arc, recv_rx_arc)
    }

    /// Get the receiver channel sender for forwarding incoming packets
    pub fn recv_tx(&self) -> Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>> {
        self.recv_tx.clone()
    }
}

#[async_trait::async_trait]
impl node::tun::PacketForwarder for Libp2pPacketForwarder {
    async fn send_packet(&self, packet: &[u8]) -> Result<()> {
        self.send_tx
            .send(packet.to_vec())
            .map_err(|e| anyhow::anyhow!("Failed to send packet: {}", e))?;
        Ok(())
    }

    async fn recv_packet(&mut self) -> Result<Vec<u8>> {
        let mut rx = self.recv_rx.lock().await;
        // Use timeout to avoid blocking forever
        match tokio::time::timeout(tokio::time::Duration::from_millis(100), rx.recv()).await {
            Ok(Some(packet)) => Ok(packet),
            Ok(None) => Err(anyhow::anyhow!("Channel closed")),
            Err(_) => Err(anyhow::anyhow!("Timeout waiting for packet")),
        }
    }
}
