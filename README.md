# CrypRQ: Post-Quantum, Zero-Trust VPN

CrypRQ explores hybrid (Kyber768 + X25519) handshakes, 5-minute “ransom timer” key rotation, and libp2p QUIC transport. Today the workspace delivers the cryptography and peer-to-peer control plane; a full userspace VPN tunnel is still under construction.

## Contact & SPDX

- © 2025 Thor Thor  
- Contact: [codethor@gmail.com](mailto:codethor@gmail.com)  
- LinkedIn: [https://www.linkedin.com/in/thor-thor0](https://www.linkedin.com/in/thor-thor0)  
- SPDX-License-Identifier: MIT

> **Status:** Prototype. The CLI generates post-quantum keys, rotates them on a schedule, and can listen or dial peers over QUIC. No packet forwarding yet.

## Features

- Hybrid Kyber768 + X25519 secrets via `pqcrypto-kyber` and `x25519-dalek`
- Asynchronous key rotation task that burns the previous key pair every 300 s
- libp2p QUIC transport with mDNS discovery stubs
- `no_std` crypto crate suitable for embedded usage
- Reproducible build scripts (musl + Docker)

## Quick Start

```bash
# prerequisites
rustup toolchain install 1.83.0
cd cryprq

# start a listener (prints its multiaddr)
cargo run --release -- --listen /ip4/0.0.0.0/udp/9001/quic-v1

# in another shell, dial the listener
cargo run --release -- --peer /ip4/127.0.0.1/udp/9001/quic-v1
```

The CLI:

- requires exactly one of `--listen` or `--peer`;
- spawns `start_key_rotation` to refresh keys on a 5-minute interval;
- logs new listen addresses or terminates once a dial succeeds.

## Architecture

```
workspace/
├── cli     # cryprq binary: argument parsing, key rotation background task, listener/dialer entry-points
├── crypto  # no_std Kyber helpers + hybrid handshake struct
├── p2p     # libp2p QUIC swarm setup and mDNS behaviour
└── node    # placeholder for future data-plane / tunnel logic
```

- **Key store:** `tokio::sync::RwLock<Option<(KyberPublicKey, KyberSecretKey)>>`
- **Networking:** `libp2p::Swarm` built with QUIC transport and dummy + mDNS behaviour
- **Logging:** use `RUST_LOG=info` (or similar) to see rotation events

## Build

### Standard Cargo build

```bash
cargo build --release
./target/release/cryprq --help
```

### Maintain SPDX headers

If you add or rename files, keep SPDX/contact headers consistent:

```bash
bash scripts/add-headers.sh
```

The script is idempotent and respects shebangs, XML declarations, and the required MIT license tag.

### Static musl binary (Linux)

```bash
./scripts/build-linux.sh
# → target/x86_64-unknown-linux-musl/release/cryprq
```

### Native macOS binary

```bash
./scripts/build-macos.sh
# → target/release/cryprq
```

### Docker image

```bash
docker build -t cryprq-cli -f Dockerfile .
docker run --rm cryprq-cli --help
```

## Manual Checks (CI currently disabled)

```bash
cargo test --release
cargo clippy --all-targets --all-features -- -D warnings
# optional but recommended
cargo install cargo-audit
cargo audit
./scripts/docker_vpn_test.sh          # listener/dialer smoke test in Docker
```

GitHub Actions now runs:
- `CI` on every push/PR (fmt, clippy, tests)
- `Security Audit` weekly and on push/PR (cargo audit, cargo deny)
- `CodeQL` weekly and on push/PR (static analysis for Rust)

Please still run the local checks above before merging changes.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). In short:

- SSH-sign commits when possible.
- Add SPDX headers to new Rust files.
- Document which manual checks you ran in your PR description.

## License

Dual-licensed under Apache 2.0 and MIT. See [LICENSE](LICENSE) for the MIT terms, [LICENSE-APACHE](LICENSE-APACHE) for Apache 2.0, and [DUAL_LICENSE.md](DUAL_LICENSE.md) for details.

## Acknowledgments

- [`libp2p`](https://libp2p.io/) for the modular P2P stack.
- [`pqcrypto`](https://crates.io/crates/pqcrypto-kyber) maintainers for the Kyber implementation.
- [`tokio`](https://tokio.rs/) for powering the async runtime.

If you experiment with CrypRQ, please open an issue or PR—we’d love to hear what you build.
