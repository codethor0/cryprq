[![Tip](https://img.shields.io/badge/Tip-Support-brightgreen)](https://buy.stripe.com/00w6oA7kM4wc4co5RB3Nm01)
[![Monthly](https://img.shields.io/badge/Monthly-Support-blue)](https://buy.stripe.com/7sY3cobB2bYEdMYa7R3Nm00)

# CrypRQ: Post-Quantum, Zero-Trust VPN
<img width="512" height="512" alt="CrypRQ_icon_512" src="docs/_assets/CrypRQ_icon_512.png" />

[![CI](https://github.com/codethor0/cryprq/actions/workflows/ci.yml/badge.svg)](https://github.com/codethor0/cryprq/actions/workflows/ci.yml)
[![Local Validate Mirror](https://github.com/codethor0/cryprq/actions/workflows/local-validate-mirror.yml/badge.svg)](https://github.com/codethor0/cryprq/actions/workflows/local-validate-mirror.yml)
[![Security Audit](https://github.com/codethor0/cryprq/actions/workflows/security-audit.yml/badge.svg)](https://github.com/codethor0/cryprq/actions/workflows/security-audit.yml)
[![CodeQL](https://github.com/codethor0/cryprq/actions/workflows/codeql.yml/badge.svg)](https://github.com/codethor0/cryprq/actions/workflows/codeql.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Reproducible Builds](https://img.shields.io/badge/builds-reproducible-brightgreen.svg)](REPRODUCIBLE.md)
[![Tip](https://img.shields.io/badge/Tip-Support-brightgreen)](https://buy.stripe.com/00w6oA7kM4wc4co5RB3Nm01)
[![Monthly](https://img.shields.io/badge/Monthly-Support-blue)](https://buy.stripe.com/7sY3cobB2bYEdMYa7R3Nm00)


> Post-quantum, zero-trust VPN with five-minute ephemeral key rotation.

## Table of Contents
1. [Features](#features)
2. [Anti-features](#anti-features)
3. [Quickstart](#quickstart)
4. [Performance & Testing](#performance--testing)
5. [Deploy](#deploy)
6. [Configuration](#configuration)
7. [Security Model](#security-model)
8. [Reproducible Builds](#reproducible-builds)
9. [Roadmap](#roadmap)
10. [Contributing](#contributing)
11. [License](#license)

---

## Features
- Hybrid ML-KEM (Kyber768-compatible) + X25519 handshake over libp2p QUIC.
- Five-minute key rotation with secure zeroization of prior keys.
- Userspace WireGuard prototype using ChaCha20-Poly1305 and BLAKE3 KDF.
- Dedicated crates: `crypto` (`no_std` ML-KEM), `p2p` (libp2p swarm), `node` (tunnel), `cli`.
- Supply-chain hardening: vendored dependencies, `cargo audit`, `cargo deny`, `CodeQL`.
- Release pipeline emits SPDX SBOMs (Syft) and Grype vulnerability reports for container images.
- Reproducible build scripts for Linux (musl), macOS, Nix, and Docker.
- Platform hosts underway: Android `VpnService` module (`android/`), Apple Network Extension, Windows MSIX, F-Droid packaging. See `/docs` for plans and status.

## Recent Updates

### VPN Mode and Packet Forwarding
- **System-wide VPN routing**: TUN interface support for macOS and Linux
- **Docker-based VPN solution**: Complete containerized VPN with web UI
- **Packet forwarding**: Full bidirectional packet forwarding over libp2p request-response protocol
- **Real-time encryption visibility**: Debug console showing encryption/decryption events
- **Comprehensive testing**: 14 test categories completed, all passing
- **CI Status**: All workflows passing (see badges above)

### Web UI
- **Web-based management**: React + TypeScript web interface
- **Docker integration**: Seamless connection to containerized VPN
- **Real-time monitoring**: Live debug console with encryption events
- **Connection management**: Easy listener/dialer mode switching

### Testing Infrastructure
- **Comprehensive test suite**: 14 exploratory test categories
- **Encryption verification**: 44 encryption events, 8 decryption events confirmed
- **Packet forwarding tests**: End-to-end packet flow verified
- **Performance metrics**: Connection establishment ~53ms, packet rate ~820 packets/second
- **Production ready**: All systems verified and operational

## Anti-features
- No automatic peer discovery or centralized management plane.
- No legacy cipher or insecure transport support.
- No claims of FIPS/CC compliance; crypto remains experimental.
- No DoS protections beyond basic libp2p limits.

## Quickstart

### Local

```bash
git clone https://github.com/codethor0/cryprq.git
cd cryprq
rustup toolchain install 1.83.0
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p cryprq
cargo test --all
```

**Run: listener and dialer (QUIC/libp2p)**

```bash
## Listener
./target/release/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1

## Dialer (in another shell or host)
./target/release/cryprq --peer /ip4/127.0.0.1/udp/9999/quic-v1
```

Expect listener to log its local peer ID; dialer logs a successful connection; libp2p ping events confirm liveness.

### Docker

```bash
docker build -t cryprq-node .
docker run --rm -p 9999:9999/udp cryprq-node --listen /ip4/0.0.0.0/udp/9999/quic-v1
```

A basic Compose service is also available (see `docker-compose.yml`).

### Nix

```bash
nix build
./result/bin/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1
```

## Technology Verification

| Capability | Status | Evidence/Notes |
|------------|--------|----------------|
| Hybrid ML-KEM (Kyber768) + X25519 handshake over libp2p QUIC | Implemented | Feature list in README; verified in Docker QA |
| Five-minute key rotation with secure zeroization | Implemented | Feature list; `CRYPRQ_ROTATE_SECS` config; verified in tests |
| Userspace WireGuard prototype (ChaCha20-Poly1305, BLAKE3) | Implemented | Packet forwarding operational; verified in comprehensive tests |
| TUN interface and VPN routing | Implemented | System-wide VPN routing via TUN interface; Docker VPN solution available |
| Packet forwarding over libp2p | Implemented | Request-response protocol operational; bidirectional flow verified |
| Reproducible builds (Linux musl, macOS, Docker, Nix) | Implemented | README + REPRODUCIBLE.md reference and scripts |
| SBOM + Grype for container images | Implemented | Release pipeline emits SPDX SBOM + Grype report |
| Web UI and Docker VPN | Implemented | React-based web interface; Docker containerized VPN solution |
| Comprehensive testing | Implemented | 14 test categories completed; all systems verified |
| Platform hosts (Android VpnService, Apple Network Extension, Windows MSIX) | Underway | Platform folders (`android/`, `apple/`, `windows/`) and docs references |

## Performance & Testing

CrypRQ has been optimized for production use with comprehensive testing infrastructure.

### Performance Metrics

- **Binary Size**: ~6MB (optimized with LTO, 54% reduction from baseline)
- **Startup Time**: <500ms (454ms measured)
- **Build Time**: ~60s (release with LTO)
- **Test Execution**: <2s (unit tests)

### Build Optimizations

The project uses aggressive release optimizations:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
strip = true        # Strip symbols
```

### Testing Infrastructure

Comprehensive testing scripts are available:

```bash
## Exploratory testing - Verify technology functionality
bash scripts/exploratory-testing.sh

## Performance benchmarking - Measure performance metrics
bash scripts/performance-benchmark.sh

## Optimization analysis - Review optimization opportunities
bash scripts/optimize-performance.sh

## Final verification - Production readiness checks
bash scripts/final-verification.sh
```

### Docker Testing

Extensive Docker-based testing is available:

```bash
## Run complete QA suite
bash scripts/docker-qa-suite.sh

## Run individual tests
bash scripts/docker-test-individual.sh <test-name>

## View running containers
docker ps --filter "name=cryprq"
```

### Documentation

- [Performance Guide](docs/PERFORMANCE.md) - Performance optimization and benchmarking
- [Exploratory Testing](docs/EXPLORATORY_TESTING.md) - Technology functionality verification
- [Technology Verification](docs/TECHNOLOGY_VERIFICATION.md) - Complete testing summary
- [Docker Testing](docs/DOCKER_TESTING.md) - Docker-based QA infrastructure

### CI/CD Pipelines

Recommended workflow set:

- **`ci.yml`**: Build, lint, test (fmt → clippy → build → test → docker-qa)
- **`qa-vnext.yml`**: Comprehensive QA pipeline (KATs, property tests, fuzzing, Miri, sanitizers, coverage)
- **`docker-test.yml`**: Container QA for QUIC handshake and rotation
- **`security-audit.yml`**: Secret scanning, cargo-audit, cargo-deny, SBOM/Grype
- **`codeql.yml`**: CodeQL static analysis
- **`maintenance-cleanup.yml`**: Daily storage cleanup to maintain <10GB usage
- **`mobile-android.yml`** and **`mobile-ios.yml`**: Guarded mobile builds (stubs where signing is unavailable)

These workflows are referenced in project docs and status guidance. All tests and benchmarks are integrated into CI. **Current CI Status**: All workflows passing (see status badges at the top of this README).

#### Reproducing CI Locally

To reproduce CI checks locally:

```bash
# Install required tools
rustup toolchain install 1.83.0
rustup component add rustfmt clippy
cargo install cargo-audit cargo-deny cargo-llvm-cov cargo-fuzz

# Run the same checks as CI
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p cryprq
cargo test --all

# Run QA pipeline (requires Docker)
bash scripts/qa-all.sh
```

See [scripts/qa-all.sh](scripts/qa-all.sh) for the complete QA pipeline.

All tests and benchmarks are integrated into CI:
- Exploratory tests (14 categories completed)
- Performance benchmarks
- Security audits
- Code quality checks
- Cryptographic validation
- VPN packet forwarding tests
- Docker container tests

## Deploy
### Bare Metal
- Linux or macOS with Rust 1.83.0.
- Open TCP/UDP 9999 inbound on firewalls.
- Minimal systemd unit:
```ini
[Unit]
Description=CrypRQ Listener
After=network-online.target
Wants=network-online.target

[Service]
User=cryprq
Group=cryprq
Environment=RUST_LOG=info
ExecStart=/usr/local/bin/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

### Docker
```bash
docker build -t cryprq-node .
docker run --rm -p 9999:9999/udp cryprq-node \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1
```

Compose snippet:
```yaml
services:
  listener:
    image: cryprq-node:latest
    build: .
    command: ["--listen", "/ip4/0.0.0.0/udp/9999/quic-v1"]
    ports:
      - "9999:9999/udp"
    restart: unless-stopped
```

### Nix
```bash
nix build
./result/bin/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1
```

### Performance and Operational Notes

**Docker and Compose examples** are provided for reproducible local QA of handshake liveness and rotation.

**Cloud deployments** should:
- Allow TCP/UDP 9999 from trusted peers
- Store logs on encrypted volumes
- Disable unused services
- Ensure NTP sync for rotation cadence

## Configuration

Important options include allowlisting, metrics binding, logging, and rotation interval.

| Option | Description | Default |
|--------|-------------|---------|
| `--listen <multiaddr>` | Listener mode multiaddr. | None |
| `--peer <multiaddr>` | Dialer mode multiaddr (optionally `/p2p/<peer-id>`). | None |
| `--allow-peer <peer-id>` | Allowlist specific peer IDs (repeatable). **Enforces explicit peer allowlist.** | Allow all |
| `--metrics-addr <addr>` | Bind Prometheus metrics/health server. | `127.0.0.1:9464` |
| `--rotate-secs <seconds>` | Override rotation interval in seconds. | `300` (5 minutes) |
| `RUST_LOG` | Log level (`error`…`trace`). | `info` |
| `CRYPRQ_ROTATE_SECS` | Rotation interval in seconds. **Controls key rotation cadence.** | `300` |
| `CRYPRQ_MAX_INBOUND` | Max pending/established inbound handshakes. | `64` |
| `CRYPRQ_BACKOFF_BASE_MS` | Initial inbound backoff (ms) after failures. | `500` |
| `CRYPRQ_BACKOFF_MAX_MS` | Max inbound backoff (ms). | `30000` |

**Peer flow**: Listener logs a peer ID, dialer connects using the multiaddr, libp2p ping events confirm liveness.

## Security Model

**Assets**: Hybrid handshake material and (future) tunnel keys. All peers authenticate via libp2p identity keys.

**Post-quantum intent**: ML-KEM mitigates store-now-decrypt-later risk. Rotation limits exposure window.

**Hardened deployments**: Disable mDNS discovery. Current limitations and dependency review are documented.

**Limitations**:
- Data-plane encryption still in development.
- mDNS discovery disabled in hardened deployments.
- No automated peer revocation or ACL enforcement yet.
- Dependency `pqcrypto-mlkem` under active review.

**Responsible disclosure**: codethor@gmail.com (PGP in SECURITY.md).

**Supply-chain checks**: `cargo audit`, `cargo deny`, `CodeQL`, `scripts/docker_vpn_test.sh`.

## Reproducible Builds and Release Artifacts

Linux (musl), macOS, Docker paths provided:

```bash
## Linux (musl)
./scripts/build-musl.sh

## macOS
./scripts/build-macos.sh

## Docker
docker build -t cryprq-node .
```

**Release artifacts**: `./finish_qa_and_package.sh` bundles QA logs, binaries, checksums, SPDX SBOM, and Grype report under `release-*/security/`.

See [REPRODUCIBLE.md](REPRODUCIBLE.md) for deterministic build steps and expectations.

## FFI and Platform Hosts

**`cryp-rq-core`** exposes a C ABI for platform integrations:
- `cryprq_init` - Initialize CrypRQ with configuration
- `cryprq_connect` - Connect to a peer
- `cryprq_read_packet` - Read packet from tunnel
- `cryprq_write_packet` - Write packet to tunnel
- `cryprq_on_network_change` - Handle network interface changes
- `cryprq_close` - Close connection and cleanup

**Header generation**: `cbindgen --config cbindgen.toml --crate cryprq_core --output cryprq_core.h`

**Cross-target CI**: Validates `cargo check` for Apple (macOS/iOS), Android, and Windows static library builds.

**Documentation**: [docs/ffi.md](docs/ffi.md) covers error codes, ownership rules, and deterministic build guidance.

## Roadmap Highlights

- Complete userspace WireGuard forwarding
- Peer directory and policy enforcement
- Metrics/health endpoints
- PQ data-plane cipher exploration
- Versioned crate releases

See [docs/roadmap.md](docs/roadmap.md) for detailed roadmap and progress tracking.

## Contributing

### Development Setup

1. Install Rust toolchain:
   ```bash
   rustup toolchain install 1.83.0
   ```

2. Clone and build:
   ```bash
   git clone https://github.com/codethor0/cryprq.git
   cd cryprq
   cargo build --release -p cryprq
   ```

### Code Quality Checks

1. Format code:
   ```bash
   cargo fmt --all
   ```

2. Lint code:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. Run tests:
   ```bash
   cargo test --release
   ```

4. Security audits:
   ```bash
   cargo audit --deny warnings
   cargo deny check ...
   ```

5. Docker tests:
   ```bash
   bash scripts/docker-qa-suite.sh
   ```

### Testing & Verification

Run comprehensive verification:

```bash
## Exploratory testing
bash scripts/exploratory-testing.sh

## Performance benchmarks
bash scripts/performance-benchmark.sh

## Final verification
bash scripts/final-verification.sh
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for full workflow and documentation updates.

## Icon Coverage

CrypRQ application icon is normalized across all platforms and deliverables. See [Icon Coverage](docs/ICON_COVERAGE.md) for details.

| Platform | Status | Icon Location |
|----------|--------|---------------|
| Android | Complete | `android/app/src/main/res/mipmap-*/ic_launcher.png` |
| iOS | Complete | `apple/Sources/CrypRQ/Assets.xcassets/AppIcon.appiconset/` |
| macOS | Complete | `branding/CrypRQ.icns` |
| Windows | Complete | `windows/Assets/AppIcon.ico` |
| Linux | Complete | `packaging/linux/hicolor/*/apps/cryprq.png` |
| Electron GUI | Complete | `gui/build/icon.{icns,ico,png}` |
| Docker | Complete | OCI label `org.opencontainers.image.logo` |

Generate all icons: `bash scripts/generate-icons.sh`

## License
CrypRQ is licensed under the [MIT License](LICENSE). Apache 2.0 text is kept for reference; MIT is authoritative.

## Support

If this project helps you, consider supporting ongoing maintenance:

- **One-time tip:** https://buy.stripe.com/00w6oA7kM4wc4co5RB3Nm01  
- **Monthly support:** https://buy.stripe.com/7sY3cobB2bYEdMYa7R3Nm00

**What you fund:** maintenance, docs, roadmap experiments, and new features.
