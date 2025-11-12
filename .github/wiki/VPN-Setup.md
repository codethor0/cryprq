# VPN Setup Guide

Complete guide for setting up CrypRQ VPN in various deployment scenarios.

## Docker VPN Solution

The recommended approach for macOS and easy deployment.

### Prerequisites

- Docker Desktop installed and running
- Network access

### Setup Steps

1. **Start VPN Container:**
   ```bash
   docker-compose -f docker-compose.vpn.yml up -d
   ```

2. **Start Web Server:**
   ```bash
   USE_DOCKER=true npm start --prefix web/server
   ```

3. **Access Web UI:**
   - Open http://localhost:8787
   - Select listener or dialer mode
   - Enable VPN mode
   - Click Connect

### Verification

- Check container logs: `docker logs cryprq-vpn`
- Verify TUN interface: `docker exec cryprq-vpn ip addr show cryprq0`
- Monitor encryption events in web UI debug console

## Native VPN Mode

For Linux and macOS with proper permissions.

### Linux

```bash
# Build with VPN support
cargo build --release --target x86_64-unknown-linux-musl

# Run with VPN mode
sudo ./target/x86_64-unknown-linux-musl/release/cryprq \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1 --vpn
```

### macOS

```bash
# Build
cargo build --release --target aarch64-apple-darwin

# Run with VPN mode
./target/aarch64-apple-darwin/release/cryprq \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1 --vpn
```

Note: Full system-wide routing on macOS requires Network Extension framework.

## Configuration

### Environment Variables

- `RUST_LOG`: Log level (error, warn, info, debug, trace)
- `CRYPRQ_ROTATE_SECS`: Key rotation interval (default: 300)
- `CRYPRQ_MAX_INBOUND`: Max inbound connections (default: 64)

### CLI Options

- `--listen <multiaddr>`: Start listener
- `--peer <multiaddr>`: Connect to peer
- `--vpn`: Enable VPN mode
- `--tun-name <name>`: TUN interface name (default: cryprq0)
- `--tun-address <ip>`: TUN interface IP (default: 10.0.0.1/24)

## Troubleshooting

See [Troubleshooting Guide](Troubleshooting) for common issues and solutions.

