# CrypRQ: Post-Quantum, Zero-Trust VPN

CrypRQ is a Rust-based, post-quantum VPN solution that implements ephemeral key rotation and utilizes the Kyber768 cryptographic algorithm for secure handshakes. This project aims to provide a secure and efficient way to establish VPN connections while ensuring that keys are regularly rotated to enhance security.

## Features

- Post-quantum cryptography using Kyber768 KEM
- Ephemeral key rotation (every 5 minutes)
- libp2p-based peer-to-peer networking
- QUIC transport protocol
- mDNS local network discovery
- Zero-trust architecture
- Static binary builds for Linux and macOS

## Requirements

- Rust toolchain 1.83.0 or later
- Docker (optional, for containerized builds)

## Project Structure

The project is organized as a Rust workspace with the following modules:

- **crypto**: Cryptographic functions and types using Kyber768 and X25519
- **p2p**: Peer-to-peer networking using libp2p
- **node**: Core VPN node implementation
- **cli**: Command-line interface for interacting with VPN nodes

## Building

### Linux (Static Binary)

To build a static binary for Linux using musl:

```bash
./scripts/build-linux.sh
```

The binary will be located at: `target/x86_64-unknown-linux-musl/release/cryprq`

### macOS

To build for macOS:

```bash
./scripts/build-macos.sh
```

The binary will be located at: `target/release/cryprq`

### Standard Cargo Build

```bash
cargo build --release
```

The binary will be located at: `target/release/cryprq`

## Usage

### Starting a Listener

To start a node that listens for incoming connections:

```bash
cargo run --release -- --listen /ip4/0.0.0.0/udp/9001/quic-v1
```

### Connecting to a Peer

To connect to a peer:

```bash
cargo run --release -- --peer /ip4/<PEER_IP>/udp/9001/quic-v1
```

### Docker Usage

Build the Docker image:

```bash
docker build -f Dockerfile.reproducible -t cryprq-dev .
```

Run a listener node:

```bash
docker run -d --name cryprq-listener --network cryprq-test \
  cryprq-dev cargo run --release -- --listen /ip4/0.0.0.0/udp/9001/quic-v1
```

Connect to a peer:

```bash
docker run --rm --network cryprq-test \
  cryprq-dev cargo run --release -- --peer /ip4/<LISTENER_IP>/udp/9001/quic-v1
```

## Testing & Quality Checks

- **Unit / integration tests**
  ```bash
  cargo test --release
  ```

- **Clippy (optional but recommended)**
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```

- **Cargo Audit (optional)**
  ```bash
  cargo install cargo-audit
  cargo audit
  ```

- **Two-node smoke test (Docker)**
  ```bash
  ./scripts/docker_vpn_test.sh
  ```

💡 We previously ran these automatically in GitHub Actions, but workflows were removed to unblock the repository. Until automation is reinstated, please run the checks above manually before merging changes.

## Architecture

- **Transport**: QUIC over UDP for low latency and multiplexing
- **Discovery**: mDNS for local network peer discovery
- **Cryptography**: 
  - Kyber768 for post-quantum key exchange
  - X25519 for classical key exchange
  - Hybrid approach for maximum security
- **Key Rotation**: Automatic rotation every 5 minutes for forward secrecy

## Security

- Uses NIST-standardized Kyber768 for post-quantum security
- Ephemeral key rotation prevents long-term key exposure
- Zero-trust architecture - no central authority
- Keys are securely zeroized on rotation

## License

This project is dual-licensed under Apache 2.0 or MIT. You may choose either (or both) licenses at your option. See `LICENSE`, `LICENSE-APACHE`, and `LICENSE-MIT` for the full text.

## Contributing

Contributions are welcome! Please ensure all tests pass and code follows Rust best practices.
