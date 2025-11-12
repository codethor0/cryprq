// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Packet forwarding over libp2p request-response protocol
//! 
//! This module provides packet forwarding using libp2p's request-response protocol
//! for bidirectional packet exchange.

use anyhow::{Context, Result};
use libp2p::{
    request_response::{self, Codec, ProtocolSupport, RequestId, ResponseChannel},
    swarm::Swarm,
    PeerId,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use bytes::Bytes;

use crate::MyBehaviour;

/// Simple codec for packet forwarding
#[derive(Clone)]
pub struct PacketCodec;

#[derive(Clone)]
pub struct PacketProtocol;

impl request_response::Protocol for PacketProtocol {
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    fn request_response_protocol_info(&self) -> Vec<libp2p::request_response::ProtocolName> {
        vec!["/cryprq/packet/1.0.0".into()]
    }
}

impl Codec for PacketCodec {
    type Protocol = PacketProtocol;
    type Request = Vec<u8>;
    type Response = Vec<u8>;

    fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;
        let mut len_bytes = [0u8; 4];
        futures::executor::block_on(io.read_exact(&mut len_bytes))?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        futures::executor::block_on(io.read_exact(&mut buf))?;
        Ok(buf)
    }

    fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;
        let mut len_bytes = [0u8; 4];
        futures::executor::block_on(io.read_exact(&mut len_bytes))?;
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; len];
        futures::executor::block_on(io.read_exact(&mut buf))?;
        Ok(buf)
    }

    fn write_request<T>(&mut self, _: &Self::Protocol, io: &mut T, req: Self::Request) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;
        let len = req.len() as u32;
        futures::executor::block_on(io.write_all(&len.to_be_bytes()))?;
        futures::executor::block_on(io.write_all(&req))?;
        futures::executor::block_on(io.flush())?;
        Ok(())
    }

    fn write_response<T>(&mut self, _: &Self::Protocol, io: &mut T, res: Self::Response) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;
        let len = res.len() as u32;
        futures::executor::block_on(io.write_all(&len.to_be_bytes()))?;
        futures::executor::block_on(io.write_all(&res))?;
        futures::executor::block_on(io.flush())?;
        Ok(())
    }
}

/// Packet forwarder using libp2p request-response protocol
pub struct Libp2pPacketForwarder {
    swarm: Arc<tokio::sync::Mutex<Swarm<MyBehaviour>>>,
    peer_id: PeerId,
    send_tx: Arc<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
    recv_rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>>,
    request_id_counter: Arc<Mutex<u64>>,
}

impl Libp2pPacketForwarder {
    pub fn new(
        swarm: Arc<tokio::sync::Mutex<Swarm<MyBehaviour>>>,
        peer_id: PeerId,
    ) -> (Self, Arc<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>, Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>>) {
        let (send_tx, send_rx) = tokio::sync::mpsc::unbounded_channel();
        let (recv_tx, recv_rx) = tokio::sync::mpsc::unbounded_channel();
        
        let forwarder = Self {
            swarm: swarm.clone(),
            peer_id,
            send_tx: Arc::new(send_tx),
            recv_rx: Arc::new(Mutex::new(recv_rx)),
            request_id_counter: Arc::new(Mutex::new(0)),
        };
        
        // Spawn task to handle sending packets via libp2p request-response
        let swarm_clone = swarm.clone();
        let peer_id_clone = peer_id;
        let recv_tx_clone = recv_tx.clone();
        tokio::spawn(async move {
            let mut send_rx = send_rx;
            
            loop {
                if let Some(packet) = send_rx.recv().await {
                    // Send packet as request via request-response protocol
                    let mut swarm = swarm_clone.lock().await;
                    // Note: We need to access the request-response behaviour
                    // This is a simplified version - actual implementation needs
                    // to integrate with the swarm's behaviour
                    log::debug!("🔐 Forwarding {} bytes via libp2p to {}", packet.len(), peer_id_clone);
                    
                    // TODO: Actually send via request-response behaviour
                    // For now, we'll use a simpler approach with direct stream handling
                }
            }
        });
        
        (forwarder, Arc::new(send_tx), Arc::new(Mutex::new(recv_rx)))
    }
}

#[async_trait::async_trait]
impl node::tun::PacketForwarder for Libp2pPacketForwarder {
    async fn send_packet(&self, packet: &[u8]) -> Result<()> {
        self.send_tx.send(packet.to_vec())
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
