// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

//! Packet forwarding over libp2p streams
//! 
//! This module provides packet forwarding using libp2p's encrypted streams.
//! For now, this is a stub implementation that logs packets.
//! Full libp2p stream integration will be added incrementally.

use anyhow::{Context, Result};
use libp2p::{
    swarm::Swarm,
    PeerId,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::MyBehaviour;

/// Packet forwarder using libp2p streams
/// 
/// This is a stub implementation that will be enhanced with full
/// libp2p stream-based packet forwarding.
pub struct Libp2pPacketForwarder {
    swarm: Arc<tokio::sync::Mutex<Swarm<MyBehaviour>>>,
    peer_id: PeerId,
    send_tx: Arc<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
    recv_rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>>,
}

impl Libp2pPacketForwarder {
    pub fn new(
        swarm: Arc<tokio::sync::Mutex<Swarm<MyBehaviour>>>,
        peer_id: PeerId,
    ) -> (Self, Arc<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>, Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>>) {
        let (send_tx, mut send_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_recv_tx, recv_rx) = tokio::sync::mpsc::unbounded_channel();
        
        let send_tx_arc = Arc::new(send_tx.clone());
        let recv_rx_arc = Arc::new(Mutex::new(recv_rx));
        
        let forwarder = Self {
            swarm: swarm.clone(),
            peer_id,
            send_tx: send_tx_arc.clone(),
            recv_rx: recv_rx_arc.clone(),
        };
        
        // Spawn task to handle sending packets via libp2p
        let swarm_clone = swarm.clone();
        let peer_id_clone = peer_id;
        tokio::spawn(async move {
            loop {
                if let Some(packet) = send_rx.recv().await {
                    // Log packet forwarding
                    log::debug!("🔐 Forwarding {} bytes via libp2p to {}", packet.len(), peer_id_clone);
                    
                    // TODO: Send packet via libp2p stream
                    // This requires integrating with libp2p's stream handling
                    // For now, we log that packets are ready to be forwarded
                    // The actual stream integration will be added in a follow-up
                }
            }
        });
        
        (forwarder, send_tx_arc, recv_rx_arc)
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
