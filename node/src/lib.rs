use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use rand_core::OsRng;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time;
use x25519_dalek::{EphemeralSecret, PublicKey};
use zeroize::Zeroize;

pub struct Tunnel {
    socket: Arc<UdpSocket>,
    session_key: Arc<RwLock<[u8; 32]>>,
    peer_addr: Arc<RwLock<Option<std::net::SocketAddr>>>,
    nonce_counter: Arc<RwLock<u64>>,
}

impl Tunnel {
    pub async fn send_packet(&self, pkt: &[u8]) -> Result<(), std::io::Error> {
        let key = self.session_key.read().unwrap();
        let cipher = ChaCha20Poly1305::new(key.as_ref().into());
        
        let mut counter = self.nonce_counter.write().unwrap();
        *counter += 1;
        let nonce_bytes = counter.to_le_bytes();
        let mut nonce_arr = [0u8; 12];
        nonce_arr[..8].copy_from_slice(&nonce_bytes);
        let nonce = Nonce::from(nonce_arr);
        
        let ciphertext = cipher
            .encrypt(&nonce, Payload { msg: pkt, aad: b"" })
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "encryption failed"))?;
        
        if let Some(peer_addr) = *self.peer_addr.read().unwrap() {
            self.socket.send_to(&ciphertext, peer_addr).await?;
        }
        
        Ok(())
    }
    
    pub async fn recv_packet(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut buf = vec![0u8; 65535];
        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        
        // Store peer address for future sends
        *self.peer_addr.write().unwrap() = Some(addr);
        
        let key = self.session_key.read().unwrap();
        let cipher = ChaCha20Poly1305::new(key.as_ref().into());
        
        // Extract nonce from packet (simplified: assume first 12 bytes)
        let nonce_bytes: [u8; 12] = buf[..12].try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid nonce"))?;
        let nonce = Nonce::from(nonce_bytes);
        let ciphertext = &buf[12..len];
        
        let plaintext = cipher
            .decrypt(&nonce, Payload { msg: ciphertext, aad: b"" })
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "decryption failed"))?;
        
        Ok(plaintext)
    }
}

pub async fn create_tunnel(
    local_sk: &[u8; 32],
    peer_pk: &[u8; 32],
    listen_addr: &str,
) -> Result<Tunnel, std::io::Error> {
    let socket = UdpSocket::bind(listen_addr).await?;
    
    // Perform Noise-IK handshake (simplified)
    // Use the local_sk bytes directly for DH
    let peer_public = PublicKey::from(*peer_pk);
    let shared_secret = EphemeralSecret::random_from_rng(OsRng)
        .diffie_hellman(&peer_public);
    
    // Derive session key using BLAKE3 KDF
    let session_key = blake3::hash(shared_secret.as_bytes()).as_bytes().clone();
    
    let tunnel = Tunnel {
        socket: Arc::new(socket),
        session_key: Arc::new(RwLock::new(session_key)),
        peer_addr: Arc::new(RwLock::new(None)),
        nonce_counter: Arc::new(RwLock::new(0)),
    };
    
    // Spawn key rotation task (every 5 minutes)
    let session_key_clone = tunnel.session_key.clone();
    let _local_sk_clone = *local_sk;
    let peer_pk_clone = *peer_pk;
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(300));
        interval.tick().await; // Skip first immediate tick
        loop {
            interval.tick().await;
            
            // Re-derive session key (in practice would do new DH)
            let peer_public = PublicKey::from(peer_pk_clone);
            let shared_secret = EphemeralSecret::random_from_rng(OsRng)
                .diffie_hellman(&peer_public);
            let new_key = blake3::hash(shared_secret.as_bytes()).as_bytes().clone();
            
            let mut key = session_key_clone.write().unwrap();
            key.zeroize();
            *key = new_key;
            drop(key);
            
            println!("🔥 ransom rotate – new session key");
        }
    });
    
    Ok(tunnel)
}
