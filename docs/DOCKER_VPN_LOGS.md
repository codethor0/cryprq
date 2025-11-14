# CrypRQ Docker VPN Logs Guide

## Overview

This guide explains how to monitor and interpret logs when running CrypRQ VPN via Docker Compose.

## Starting the VPN Stack

```bash
docker compose -f docker-compose.vpn.yml up --build
```

This starts two containers:
- `cryprq-listener` - Listens on UDP port 9999
- `cryprq-dialer` - Connects to the listener

## Viewing Logs

### All Containers

```bash
docker compose -f docker-compose.vpn.yml logs -f
```

### Specific Container

```bash
# Listener logs
docker compose -f docker-compose.vpn.yml logs -f cryprq-listener

# Dialer logs
docker compose -f docker-compose.vpn.yml logs -f cryprq-dialer
```

## Expected Log Sequence

### Listener Container

```
INFO  cryprq::p2p event=listener_starting addr=/ip4/0.0.0.0/udp/9999/quic-v1
INFO  cryprq::p2p event=listener_ready
INFO  cryprq::node VPN MODE ENABLED - System-wide routing mode
INFO  cryprq::node Creating TUN interface for packet forwarding...
INFO  cryprq::p2p event=rotation_task_started interval_secs=300
INFO  cryprq::p2p event=handshake_complete peer_id=12D3KooW...
INFO  cryprq::p2p event=connection_established peer_id=12D3KooW...
INFO  cryprq::node Connection established with 12D3KooW... - Starting VPN packet forwarding
INFO  cryprq::node Starting packet forwarding loop - routing system traffic through encrypted tunnel
INFO  cryprq::node Packet forwarding loop started successfully
INFO  cryprq::p2p event=key_rotation
```

### Dialer Container

```
INFO  cryprq::p2p event=dialer_starting peer=/ip4/cryprq-listener/udp/9999/quic-v1
INFO  cryprq::node VPN MODE ENABLED - System-wide routing mode
INFO  cryprq::node Creating TUN interface for packet forwarding...
INFO  cryprq::p2p event=rotation_task_started interval_secs=300
INFO  cryprq::p2p event=handshake_complete peer_id=12D3KooW...
INFO  cryprq::p2p event=connection_established peer_id=12D3KooW...
INFO  cryprq::node Connected to 12D3KooW... - Starting VPN packet forwarding
INFO  cryprq::node Starting packet forwarding loop - routing system traffic through encrypted tunnel
INFO  cryprq::node Packet forwarding loop started successfully
INFO  cryprq::p2p event=key_rotation
```

## Verifying VPN Functionality

### 1. Check Connection Established

Both containers should show:
```
event=connection_established
```

### 2. Check Key Rotation

After 5 minutes (300 seconds), you should see:
```
event=key_rotation
```

### 3. Check Packet Forwarding

Both containers should show:
```
Packet forwarding loop started successfully
```

## Filtering Logs

### Using grep

```bash
# Show only handshake events
docker compose -f docker-compose.vpn.yml logs | grep "event=handshake"

# Show only rotation events
docker compose -f docker-compose.vpn.yml logs | grep "event=rotation"

# Show errors
docker compose -f docker-compose.vpn.yml logs | grep -i error
```

### Using docker logs directly

```bash
# Listener handshake events
docker logs cryprq-listener 2>&1 | grep "event=handshake"

# Dialer connection events
docker logs cryprq-dialer 2>&1 | grep "event=connection"
```

## Troubleshooting

### Listener not starting

```bash
# Check listener logs
docker compose -f docker-compose.vpn.yml logs cryprq-listener

# Common issues:
# - Port 9999 already in use
# - Permission denied (TUN interface requires NET_ADMIN)
```

### Dialer not connecting

```bash
# Check dialer logs
docker compose -f docker-compose.vpn.yml logs cryprq-dialer

# Common issues:
# - Network connectivity (check docker network)
# - Listener not ready (check listener logs first)
# - DNS resolution (use container name, not IP)
```

### No key rotation

```bash
# Verify environment variable
docker compose -f docker-compose.vpn.yml config | grep CRYPRQ_ROTATE_SECS

# Should show: CRYPRQ_ROTATE_SECS=300
```

### TUN interface errors

If you see:
```
Failed to configure TUN interface IP (may need root/admin)
```

This is expected in Docker - the TUN interface is created but IP configuration may require additional privileges. The encrypted tunnel still works.

## Stopping the VPN

```bash
docker compose -f docker-compose.vpn.yml down
```

This stops and removes both containers.

## Clean Restart

```bash
# Stop and remove containers
docker compose -f docker-compose.vpn.yml down

# Remove images (optional)
docker rmi cryprq-node:latest

# Rebuild and start
docker compose -f docker-compose.vpn.yml up --build
```

