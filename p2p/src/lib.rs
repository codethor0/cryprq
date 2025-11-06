use libp2p::{
    identity, mdns,
    swarm::{NetworkBehaviour, Swarm},
    PeerId, SwarmBuilder,
};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;
use zeroize::Zeroize;
use cryprq_crypto::make_kyber_keys;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    mdns: mdns::tokio::Behaviour,
}

pub struct Connection;

static KEYS: once_cell::sync::Lazy<Arc<RwLock<(Vec<u8>, Vec<u8>)>>> = 
    once_cell::sync::Lazy::new(|| {
        let (pk, sk) = make_kyber_keys();
        Arc::new(RwLock::new((pk.to_vec(), sk.to_vec())))
    });

pub async fn dial_peer(_peer_id: PeerId) -> Result<Connection, std::io::Error> {
    // TODO: implement actual dialing
    Ok(Connection)
}

pub fn get_current_pk() -> Vec<u8> {
    KEYS.read().unwrap().0.clone()
}

pub async fn start_key_rotation() {
    let mut interval = time::interval(Duration::from_secs(300)); // 5 minutes
    interval.tick().await; // Skip the first immediate tick
    loop {
        interval.tick().await;
        let (new_pk, new_sk) = make_kyber_keys();
        
        let mut guard = KEYS.write().unwrap();
        guard.0.zeroize();
        guard.1.zeroize();
        *guard = (new_pk.to_vec(), new_sk.to_vec());
        drop(guard);
        
        println!("🔥 ransom rotate");
    }
}

pub async fn init_swarm() -> Result<Swarm<Behaviour>, Box<dyn std::error::Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    
    let behaviour = Behaviour {
        mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?,
    };
    
    let swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| c)
        .build();
    
    Ok(swarm)
}
