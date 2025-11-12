# Getting Started with CrypRQ

This guide will help you get started with CrypRQ VPN.

## Prerequisites

- Rust toolchain 1.83.0 or later
- Docker (for containerized deployment)
- Network access for peer connections

## Installation

### From Source

```bash
git clone https://github.com/codethor0/cryprq.git
cd cryprq
rustup toolchain install 1.83.0
cargo build --release -p cryprq
```

### Docker

```bash
docker build -t cryprq-vpn .
docker-compose -f docker-compose.vpn.yml up -d
```

## Quick Start

### Basic Connection

**Listener:**
```bash
./target/release/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1
```

**Dialer:**
```bash
./target/release/cryprq --peer /ip4/127.0.0.1/udp/9999/quic-v1
```

### VPN Mode

**With VPN routing:**
```bash
./target/release/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1 --vpn
```

### Docker VPN with Web UI

```bash
# Start container
docker-compose -f docker-compose.vpn.yml up -d

# Start web server
USE_DOCKER=true npm start --prefix web/server

# Access web UI at http://localhost:8787
```

## Next Steps

- Read the [VPN Setup Guide](VPN-Setup)
- Check the [Testing Guide](Testing)
- Review [Troubleshooting](Troubleshooting) if you encounter issues

