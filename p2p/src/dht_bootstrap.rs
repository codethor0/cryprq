#[cfg(feature = "dht-bootstrap")]
pub const IPNS_KEY: &str = "/ipns/k51qzi5uqu5d...";

#[cfg(feature = "dht-bootstrap")]
pub async fn fetch_bootstrap_list() -> Vec<libp2p::Multiaddr> {
    use libp2p_kad::record::Key;
    use libp2p_kad::store::MemoryStore;
    use libp2p_kad::{Kademlia, KademliaConfig, Quorum};
    use libp2p::{identity, PeerId};
    use anyhow::Result;
    use std::str::FromStr;

    // Simulate fetching from IPNS (replace with real IPFS HTTP call in integration)
    let json = r#"["/ip4/127.0.0.1/tcp/4001", "/ip4/127.0.0.1/tcp/4002"]"#;
    let addrs: Vec<String> = serde_json::from_str(json).unwrap();
    addrs.into_iter().filter_map(|s| libp2p::Multiaddr::from_str(&s).ok()).collect()
}

#[cfg(not(feature = "dht-bootstrap"))]
pub async fn fetch_bootstrap_list() -> Vec<libp2p::Multiaddr> {
    // Fallback to mDNS (simulate)
    vec![]
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_fetch_bootstrap_list() {
        let rt = Runtime::new().unwrap();
        let addrs = rt.block_on(fetch_bootstrap_list());
        assert!(addrs.len() >= 2);
        assert!(addrs.iter().all(|a| a.to_string().contains("127.0.0.1")));
    }
}
