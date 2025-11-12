// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Packet forwarding over libp2p streams
//! 
//! This module provides packet forwarding functionality that bridges
//! TUN interface packets with libp2p encrypted streams.

use anyhow::{Context, Result};
use libp2p::{
    request_response::{self, Codec, ProtocolSupport, RequestId, ResponseChannel},
    swarm::Swarm,
    PeerId,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

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

/// Packet forwarder that uses libp2p request-response
pub struct Libp2pPacketForwarder {
    swarm: Arc<Mutex<Swarm<super::MyBehaviour>>>,
    peer_id: PeerId,
    pending_requests: Arc<Mutex<HashMap<RequestId, tokio::sync::oneshot::Sender<Vec<u8>>>>>,
    request_id_counter: Arc<Mutex<u64>>,
}

impl Libp2pPacketForwarder {
    pub fn new(swarm: Arc<Mutex<Swarm<super::MyBehaviour>>>, peer_id: PeerId) -> Self {
        Self {
            swarm,
            peer_id,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            request_id_counter: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl crate::tun::PacketForwarder for Libp2pPacketForwarder {
    async fn send_packet(&self, packet: &[u8]) -> Result<()> {
        // For now, use a simple approach: send packet as request
        // In production, we'd want bidirectional streams
        let mut swarm = self.swarm.lock().await;
        let mut counter = self.request_id_counter.lock().await;
        *counter += 1;
        let request_id = RequestId::from(*counter);
        
        // Create channel for response
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending_requests.lock().await.insert(request_id, tx);
        
        // Send request (packet)
        swarm.behaviour_mut().send_request(&self.peer_id, packet.to_vec());
        
        // Wait for response (acknowledgment)
        let _ = rx.await;
        
        Ok(())
    }

    async fn recv_packet(&mut self) -> Result<Vec<u8>> {
        // This is a simplified version - in production we'd use event loop
        // For now, return empty to avoid blocking
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(vec![])
    }
}

