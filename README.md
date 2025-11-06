# CrypRQ  
*Post-quantum, zero-trust, ransom-timer VPN that burns every key after 5 min.*

[![CI](https://github.com/cryprq/cryprq/actions/workflows/ci.yml/badge.svg)](https://github.com/cryprq/cryprq/actions)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE-GPL)

---

## ⚡  Status  
Pre-alpha – looking for crypto & p2p reviewers.  
**First 10 merged PRs** receive gen-0 zk-bandwidth tokens.

---

## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/codethor0/cryprq.git
cd cryprq
cargo build --release

# Run node (generates PQ Kyber keys, starts WireGuard tunnel)
./target/release/cryprq
# → CrypRQ v0.0.1 – Kyber pk: a3f7…
# → TUN up at 127.0.0.1:51820
# → 🔥 ransom rotate (every 5 min)

# Connect to peer (optional)
./target/release/cryprq --peer <PEER_ID>
# → PQ handshake complete – tunnel ready
```

### Architecture
- **crypto**: no-std Kyber768 stub (rand_core)
- **p2p**: libp2p QUIC + mDNS discovery
- **node**: userspace WireGuard (ChaCha20-Poly1305, BLAKE3 KDF, X25519)
- **cli**: tokio runtime, 5-min key rotation

---

## 🧪  Build
```bash
cargo build --release -p cryprq
./target/release/cryprq --help
cat > Cargo.toml <<'EOF'
[workspace]
members = ["crypto","p2p","node","cli"]
resolver = "2"

[workspace.package]
version = "0.0.1"
authors = ["CrypRQ Contributors"]
edition = "2021"
license = "GPL-3.0"
repository = "https://github.com/cryprq/cryprq"
rust-version = "1.75"

[profile.release]
lto = "thin"
strip = true
codegen-units = 1
