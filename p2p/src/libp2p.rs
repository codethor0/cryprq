// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

//! The libp2p components this crate uses, under the names the umbrella
//! `libp2p` crate gives them, and the swarm construction that the umbrella's
//! `SwarmBuilder` used to do.
//!
//! The umbrella crate is deliberately not a dependency. It lists every
//! protocol crate as an optional dependency, so Cargo records all of them in
//! `Cargo.lock`, including mDNS and DNS with their `hickory-proto` parser,
//! whether or not they are built. Depending on the components directly keeps
//! the lockfile to what is compiled.

pub use futures;
pub use libp2p_connection_limits as connection_limits;
pub use libp2p_core::{multiaddr, Multiaddr};
pub use libp2p_identity as identity;
pub use libp2p_identity::PeerId;
pub use libp2p_noise as noise;
pub use libp2p_ping as ping;
pub use libp2p_quic as quic;
pub use libp2p_request_response as request_response;
pub use libp2p_swarm as swarm;
pub use libp2p_swarm::{StreamProtocol, Swarm};
pub use libp2p_tcp as tcp;
pub use libp2p_yamux as yamux;

use libp2p_core::muxing::StreamMuxerBox;
use libp2p_core::transport::timeout::TransportTimeout;
use libp2p_core::transport::Transport;
use libp2p_core::upgrade::Version;
use std::time::Duration;

/// How long a connection attempt, including its upgrade, may take.
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Builds a swarm that speaks TCP (Noise, Yamux) and QUIC with the tokio
/// runtime: the same transports, upgrade version, order and timeout as
/// `SwarmBuilder::with_tokio().with_tcp(..).with_quic()` of libp2p 0.56.
pub fn build_swarm<B>(
    keypair: identity::Keypair,
    behaviour: B,
    configure: impl FnOnce(swarm::Config) -> swarm::Config,
) -> Result<Swarm<B>, noise::Error>
where
    B: swarm::NetworkBehaviour,
{
    let tcp_transport = tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(Version::V1Lazy)
        .authenticate(noise::Config::new(&keypair)?)
        .multiplex(yamux::Config::default())
        .map(|(peer, muxer), _| (peer, StreamMuxerBox::new(muxer)));
    let quic_transport = quic::tokio::Transport::new(quic::Config::new(&keypair))
        .map(|(peer, muxer), _| (peer, StreamMuxerBox::new(muxer)));
    let transport = tcp_transport
        .or_transport(quic_transport)
        .map(|either, _| either.into_inner());

    Ok(Swarm::new(
        TransportTimeout::new(transport, CONNECTION_TIMEOUT).boxed(),
        behaviour,
        keypair.public().to_peer_id(),
        configure(swarm::Config::with_tokio_executor()),
    ))
}
