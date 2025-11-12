# VPN Testing Guide

This guide covers testing the end-to-end VPN routing functionality of CrypRQ.

## Prerequisites

- Docker Desktop running
- Rust toolchain installed
- Network access for testing

## Quick Test

### 1. Start Docker VPN Container

```bash
# Build and start the container
docker-compose -f docker-compose.vpn.yml up -d

# Check container status
docker ps | grep cryprq-vpn

# View logs
docker logs -f cryprq-vpn
```

### 2. Connect to Container

```bash
# Get container IP
CONTAINER_IP=$(docker inspect cryprq-vpn --format='{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}')

# Connect as dialer
cargo run --bin cryprq -- --peer "/ip4/$CONTAINER_IP/udp/9999/quic-v1" --vpn
```

### 3. Verify Packet Forwarding

Check for encryption/decryption events in logs:

```bash
# Container logs
docker logs cryprq-vpn | grep -E "(ENCRYPT|DECRYPT|Forwarded)"

# Local dialer logs
# (should show in terminal output)
```

## Expected Behavior

### Connection Establishment

-  Container starts listening on port 9999/udp
-  Dialer connects successfully
-  Connection established message appears
-  Packet forwarding loop starts

### Packet Flow

-  Outgoing packets: TUN → encrypted tunnel → peer
-  Incoming packets: peer → encrypted tunnel → TUN
-  Encryption events logged (` ENCRYPT`)
-  Decryption events logged (` DECRYPT`)
-  Packet forwarding events logged (` Forwarded`)

### TUN Interface

```bash
# Check TUN interface in container
docker exec cryprq-vpn ip addr show cryprq0

# Check routing table
docker exec cryprq-vpn ip route show
```

Expected:
- TUN interface `cryprq0` created
- IP address configured (typically `10.0.0.1/24`)
- Routing table shows routes through TUN

## Troubleshooting

### Container Not Starting

```bash
# Check Docker logs
docker logs cryprq-vpn

# Verify Docker is running
docker ps

# Restart container
docker-compose -f docker-compose.vpn.yml restart
```

### Connection Timeout

- Verify container IP is correct
- Check firewall rules
- Ensure port 9999/udp is accessible
- Check container logs for errors

### No Packet Forwarding

- Verify VPN mode is enabled (`--vpn` flag)
- Check TUN interface exists: `docker exec cryprq-vpn ip addr show cryprq0`
- Verify connection is established (check logs for "Connection established")
- Check for `recv_tx` channel registration in logs

### No Encryption Events

- Verify `RUST_LOG=debug` is set
- Check logs for connection establishment
- Ensure packets are actually flowing (generate test traffic)
- Verify packet forwarding loop started successfully

## Advanced Testing

### Generate Test Traffic

```bash
# From host, ping container's TUN IP
docker exec cryprq-vpn ip addr show cryprq0 | grep "inet " | awk '{print $2}' | cut -d/ -f1

# Ping from another container or host
ping <TUN_IP>
```

### Monitor Packet Flow

```bash
# Watch container logs in real-time
docker logs -f cryprq-vpn | grep -E "(ENCRYPT|DECRYPT|packet)"

# Monitor TUN interface
watch -n 1 'docker exec cryprq-vpn ip -s link show cryprq0'
```

### Test Bidirectional Flow

1. Start listener in container (already running)
2. Connect dialer from host
3. Generate traffic from both sides
4. Verify encryption/decryption events in both directions

## Web UI Testing

The web UI provides a convenient way to test VPN functionality:

```bash
# Start web server in Docker mode
USE_DOCKER=true BRIDGE_PORT=8787 node web/server/server.mjs

# Open browser to http://localhost:8787
# Enable VPN mode checkbox
# Click Connect
# Monitor debug console for encryption events
```

## Success Criteria

 Container starts and listens on port 9999  
 Dialer connects successfully  
 TUN interface created and configured  
 Packet forwarding loop active  
 Encryption events logged (` ENCRYPT`)  
 Decryption events logged (` DECRYPT`)  
 Packets forwarded successfully (` Forwarded`)  
 Bidirectional packet flow working  

## Next Steps

Once basic packet forwarding is verified:

1. Configure system-wide routing (requires root/admin)
2. Test with real applications (browser, curl, etc.)
3. Verify all traffic routes through encrypted tunnel
4. Test performance and latency
5. Verify key rotation is working

