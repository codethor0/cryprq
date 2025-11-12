# Docker-Based VPN Setup

This document describes how to run CrypRQ VPN in a Docker container, allowing your Mac to connect to the container for system-wide VPN routing.

## Architecture

```
Mac Browser/App
    ↓
Web Bridge Server (docker-bridge.mjs)
    ↓
Docker Container (cryprq-vpn)
    ↓
Encrypted Tunnel (libp2p QUIC)
    ↓
Internet
```

The container handles:
- All encryption (ML-KEM + X25519 hybrid)
- TUN interface creation and packet forwarding
- Routing configuration
- Key rotation

Your Mac just needs to connect to the container.

## Quick Start

### 1. Start the VPN Container

```bash
# Start container
./scripts/docker-vpn-start.sh

# Or manually:
docker-compose -f docker-compose.vpn.yml up -d
```

### 2. Get Container IP

```bash
docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cryprq-vpn
```

### 3. Connect from Mac

The web UI will automatically connect to the container. Or manually:

```bash
# Connect Mac to container
./target/release/cryprq --peer /ip4/<CONTAINER_IP>/udp/9999/quic-v1 --vpn
```

## Using Docker Bridge Server

The `web/server/docker-bridge.mjs` server manages container communication:

```bash
# Start bridge server
cd web/server
node docker-bridge.mjs

# Or set environment variable to use Docker mode
export USE_DOCKER=true
npm start
```

The bridge server:
- Automatically starts/stops the container
- Streams container logs to the web UI
- Manages Mac-to-container connections
- Handles VPN mode configuration

## Container Configuration

The container runs with:
- **Privileged mode**: Required for TUN interface creation
- **NET_ADMIN capability**: Required for network configuration
- **Port mapping**: 9999/udp for QUIC, 8787 for web bridge
- **VPN mode**: Automatically enabled with `--vpn` flag

## Troubleshooting

### Container won't start

```bash
# Check logs
docker-compose -f docker-compose.vpn.yml logs

# Check if port is in use
lsof -i :9999

# Restart container
docker-compose -f docker-compose.vpn.yml restart
```

### Can't connect to container

```bash
# Verify container is running
docker ps | grep cryprq-vpn

# Get container IP
docker inspect cryprq-vpn | grep IPAddress

# Test connectivity
ping <CONTAINER_IP>
```

### TUN interface not working

The container needs privileged mode and NET_ADMIN capability. Verify:

```bash
docker inspect cryprq-vpn | grep -A 5 Privileged
docker inspect cryprq-vpn | grep -A 5 CapAdd
```

## Benefits of Docker Approach

1. **Isolation**: VPN runs in isolated container
2. **No macOS Network Extension**: Avoids macOS-specific requirements
3. **Easy Management**: Start/stop with docker-compose
4. **Portability**: Works on any platform with Docker
5. **Resource Control**: Limit CPU/memory usage
6. **Logging**: Centralized container logs

## Next Steps

- Configure routing tables to route Mac traffic through container
- Set up DNS forwarding
- Implement packet forwarding between Mac and container
- Add health checks and auto-restart

