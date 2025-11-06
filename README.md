lto = "thin"
strip = true
codegen-units = 1

# CrypRQ
[![CI](https://github.com/codethor0/cryprq/actions/workflows/ci.yml/badge.svg)](https://github.com/codethor0/cryprq/actions)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE)

Post-quantum, zero-trust VPN with ephemeral key rotation every five minutes.

CrypRQ is a post-quantum, zero-trust VPN designed for ephemeral key rotation and maximum cryptographic agility. It leverages Kyber768 for post-quantum handshakes, ChaCha20-Poly1305 for transport, and BLAKE3/X25519 for fallback. Peer discovery uses libp2p QUIC and mDNS. Keys are burned and rotated every five minutes for forward secrecy and ransomware resistance.

- **Website:** https://cryprq.org
- **Topics:** post-quantum, vpn, kyber, wireguard, libp2p, cryptography, security, reproducible-builds

## Status
Pre-alpha – seeking cryptographic & P2P reviewers.  
First ten merged PRs receive gen-0 zk-bandwidth tokens.

## Features
- Kyber768 post-quantum handshakes
- ChaCha20-Poly1305 transport
- BLAKE3 KDF & X25519 ECDH fallback
- libp2p QUIC + mDNS peer discovery
- 5-minute key burn (ransom-timer)
- Reproducible musl builds (4 MB static)

## Quick Start
```bash
git clone https://github.com/codethor0/cryprq.git
cd cryprq
cargo build --release
./target/release/cryprq
# TUN up at 127.0.0.1:51820
```


# Connect to peer (optional)
