# Docker VPN Quick Start Guide

## Overview

CrypRQ can run in a Docker container, with your Mac connecting to it. The container handles all encryption, tunneling, and routing, avoiding macOS Network Extension requirements.

## Architecture

```
Mac Browser/App → Docker Container → Encrypted Tunnel → Internet
```

## Quick Start (3 Steps)

### 1. Start Docker Container

```bash
./scripts/docker-vpn-start.sh
```

This will:
- Build the Docker image (if needed)
- Start the container with VPN capabilities
- Show container IP and connection info

### 2. Start Web Server with Docker Mode

```bash
./scripts/start-web-docker.sh
```

Or manually:
```bash
export USE_DOCKER=true
cd web/server
node server.mjs
```

### 3. Open Web UI

Open your browser to `http://localhost:8787` (or whatever port the web server is on).

The web UI will automatically:
- Connect to the Docker container
- Show container logs
- Allow you to connect as dialer to the container listener

## Testing Connection

```bash
# Test container connection
./scripts/test-docker-connection.sh

# Or manually connect
CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cryprq-vpn)
./target/release/cryprq --peer /ip4/$CONTAINER_IP/udp/9999/quic-v1
```

## Container Management

```bash
# Start container
./scripts/docker-vpn-start.sh

# Stop container
./scripts/docker-vpn-stop.sh

# View logs
docker-compose -f docker-compose.vpn.yml logs -f

# Restart container
docker-compose -f docker-compose.vpn.yml restart
```

## Troubleshooting

### Container won't start

```bash
# Check Docker is running
docker ps

# Check logs
docker-compose -f docker-compose.vpn.yml logs

# Rebuild image
docker-compose -f docker-compose.vpn.yml build --no-cache
```

### Can't connect to container

```bash
# Get container IP
docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cryprq-vpn

# Test connectivity
ping <CONTAINER_IP>

# Check container is listening
docker exec cryprq-vpn netstat -uln | grep 9999
```

### Web UI not connecting

1. Ensure container is running: `docker ps | grep cryprq-vpn`
2. Check web server is using Docker mode: `echo $USE_DOCKER`
3. Check web server logs for errors
4. Verify container IP is accessible

## Benefits

 **No macOS Network Extension** - Avoids macOS-specific requirements  
 **Isolated Environment** - VPN runs in container  
 **Easy Management** - Start/stop with docker-compose  
 **Portable** - Works on any platform with Docker  
 **Centralized Logging** - All logs in container  

## Next Steps

Once connected:
1. Container handles encryption (ML-KEM + X25519)
2. Container handles key rotation (every 5 minutes)
3. Mac connects to container for VPN routing
4. All traffic encrypted through container tunnel

See `docs/DOCKER_VPN_SETUP.md` for detailed documentation.

