/// Peer-to-peer networking for VPN tunnels
use anyhow::Result;
use libp2p::{
    identity,
    swarm::{Swarm, SwarmEvent},
    Multiaddr, PeerId,
    quic,
};
use libp2p::futures::StreamExt;

pub async fn start_listener(addr: &str) -> Result<()> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local PeerId: {peer_id}");

    let quic_config = quic::Config::new(&id_keys);
    let transport = quic::tokio::Transport::new(quic_config);

    let behaviour = libp2p::swarm::dummy::Behaviour;
    let mut swarm = Swarm::new(transport, behaviour, peer_id);

    let listen_addr: Multiaddr = addr.parse()?;
    swarm.listen_on(listen_addr)?;

    loop {
        match swarm.next().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address}");
            }
            _ => {}
        }
    }
}

pub async fn dial_peer(addr: String) -> Result<()> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local PeerId: {peer_id}");

    let quic_config = quic::Config::new(&id_keys);
    let transport = quic::tokio::Transport::new(quic_config);

    let behaviour = libp2p::swarm::dummy::Behaviour;
    let mut swarm = Swarm::new(transport, behaviour, peer_id);

    let dial_addr: Multiaddr = addr.parse()?;
    swarm.dial(dial_addr)?;

    loop {
        match swarm.next().await {
            SwarmEvent::ConnectionEstablished { peer_id: remote, .. } => {
                println!("Connected to {remote}");
                break;
            }
            SwarmEvent::OutgoingConnectionError { error, .. } => {
                anyhow::bail!("Dial error: {error}");
            }
            _ => {}
        }
    }
    Ok(())
}
