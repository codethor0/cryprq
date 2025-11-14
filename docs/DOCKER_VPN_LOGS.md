# Docker VPN Logging Guide

## Overview

This guide explains how to view and interpret CrypRQ logs when running in Docker VPN mode.

## Docker Compose Logs

The `docker-compose.vpn.yml` configuration sets `RUST_LOG=info` to show structured event logs.

### View Container Logs

```bash
# Follow logs in real-time
docker-compose -f docker-compose.vpn.yml logs -f

# Or directly
docker logs -f cryprq-vpn

# Last 50 lines
docker logs --tail 50 cryprq-vpn
```

### Expected Logs

When the container starts as a listener, you should see:

```
event=listener_starting peer_id=12D3KooW... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 transport=QUIC/libp2p
Local peer id: 12D3KooW...
event=listener_ready peer_id=12D3KooW... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 status=accepting_connections
Listening on /ip4/0.0.0.0/udp/9999/quic-v1
event=rotation_task_started interval_secs=300
🔒 VPN MODE ENABLED - System-wide routing mode
Creating TUN interface for packet forwarding...
✅ TUN interface cryprq0 configured with IP 10.0.0.1
```

When a peer connects:

```
event=handshake_complete peer_id=... direction=inbound endpoint=... encryption=ML-KEM+X25519 status=ready
event=connection_established peer_id=... transport=QUIC/libp2p encryption_active=true
✅ Connection established with ... - Starting VPN packet forwarding
🚀 Starting packet forwarding loop - routing system traffic through encrypted tunnel
```

Key rotation (every 5 minutes):

```
event=key_rotation status=success epoch=<N> duration_ms=<MS> interval_secs=300
```

## Web UI Logs

The web UI (`web/server/server.mjs`) captures and displays logs from spawned CrypRQ processes.

### Log Categories

The web UI categorizes logs into:
- **peer**: Connection, handshake, peer ID events
- **rotation**: Key rotation events
- **status**: VPN mode, TUN interface status
- **error**: Errors and failures
- **info**: General informational messages

### Structured Event Recognition

The web UI recognizes structured `event=` logs:
- `event=listener_starting` → peer category
- `event=dialer_starting` → peer category
- `event=listener_ready` → peer category
- `event=handshake_complete` → peer category
- `event=connection_established` → peer category
- `event=rotation_task_started` → rotation category
- `event=key_rotation` → rotation category
- `event=ppk_derived` → rotation category

### Viewing Logs in Web UI

1. Open `http://localhost:5173` (or your web UI URL)
2. Click "Connect" to start listener or dialer
3. Watch the "Debug Console" for real-time logs
4. Logs are color-coded by category:
   - **Green**: Peer/connection events
   - **Blue**: Rotation events
   - **Yellow**: Status messages
   - **Red**: Errors

## Environment Variables

### Container Log Level

Set in `docker-compose.vpn.yml`:
```yaml
environment:
  - RUST_LOG=info  # Options: error, warn, info, debug, trace
```

### Web Server Log Level

Set via environment variable when starting web server:
```bash
RUST_LOG=info node web/server/server.mjs
```

Default is `info` if not set.

## Log Filtering Examples

### Show Only Handshake Events

```bash
docker logs cryprq-vpn 2>&1 | grep "event=handshake_complete"
```

### Show Only Key Rotation Events

```bash
docker logs cryprq-vpn 2>&1 | grep "event=key_rotation"
```

### Show All Structured Events

```bash
docker logs cryprq-vpn 2>&1 | grep "event="
```

### Show Errors and Warnings

```bash
docker logs cryprq-vpn 2>&1 | grep -E "ERROR|WARN|error|warn|failed"
```

## Troubleshooting

### No Structured Events in Logs

**Problem**: Logs show raw output but no `event=` structured logs.

**Solution**: 
1. Verify `RUST_LOG=info` is set in container environment
2. Check that you're running the latest build with enhanced logging
3. Rebuild container: `docker-compose -f docker-compose.vpn.yml build --no-cache`

### Web UI Not Showing Logs

**Problem**: Web UI debug console is empty or not updating.

**Solution**:
1. Check browser console for errors
2. Verify EventSource connection: Check Network tab for `/events` endpoint
3. Check web server logs: `tail -f /tmp/cryprq-server.log` (if logging to file)
4. Verify CrypRQ process is running: Check process list

### Container Logs Too Verbose

**Problem**: Too many logs, hard to find important events.

**Solution**: 
1. Set `RUST_LOG=info` (default) instead of `trace` or `debug`
2. Filter logs: `docker logs cryprq-vpn 2>&1 | grep "event="`
3. Use structured event filtering (see examples above)

## Integration with Monitoring

### Prometheus Metrics

CrypRQ exposes Prometheus metrics (if metrics server is enabled):
- `handshakes_total`: Total handshake attempts
- `handshakes_success`: Successful handshakes
- `rotations_total`: Total key rotations
- `rotation_epoch`: Current rotation epoch
- `active_peers`: Number of active peer connections

### Log Aggregation

Structured `event=` logs are easy to parse with:
- **ELK Stack**: Parse `event=` field for filtering
- **Loki**: Query logs by event type
- **Splunk**: Extract structured fields
- **Grafana**: Visualize event timelines

Example LogQL query for Loki:
```logql
{container="cryprq-vpn"} |~ "event=(handshake_complete|key_rotation)"
```

## Security Notes

- **No sensitive data**: Logs never contain keys, secrets, or plaintext packet contents
- **Peer IDs only**: Logs show public peer identifiers, not private keys
- **Epoch counters**: Rotation logs show epoch numbers, not actual keys
- **Debug level**: Packet-level logs are debug-only and don't expose sensitive data

## See Also

- `docs/OPERATOR_LOGS.md` - Complete log reference for CLI mode
- `README_DOCKER_VPN.md` - Docker VPN quickstart guide
- `web/README.md` - Web UI documentation

