# CrypRQ Operator Log Reference

## Overview

This document provides a reference for interpreting CrypRQ logs to verify that the VPN is working correctly. All logs use structured `event=` format for easy parsing and filtering.

## Log Levels

- **INFO**: Critical operational events (handshake, rotation, connection status)
- **DEBUG**: Detailed packet-level events (encryption/decryption, forwarding)
- **WARN**: Non-fatal issues (privilege errors, connection retries)
- **ERROR**: Fatal errors requiring attention

## Key Events

### Listener Startup

When starting a listener, you should see:

```
event=listener_starting peer_id=<PEER_ID> listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 transport=QUIC/libp2p
Local peer id: <PEER_ID>
event=listener_ready peer_id=<PEER_ID> listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 status=accepting_connections
Listening on /ip4/0.0.0.0/udp/9999/quic-v1
```

**What this means**: Listener is ready to accept connections.

### Dialer Startup

When starting a dialer, you should see:

```
event=dialer_starting peer_id=<PEER_ID> target_addr=/ip4/127.0.0.1/udp/9999/quic-v1 transport=QUIC/libp2p
Local peer id: <PEER_ID>
```

**What this means**: Dialer is attempting to connect to peer.

### Handshake Completion

When a connection is established (both listener and dialer), you should see:

```
event=handshake_complete peer_id=<PEER_ID> direction=<inbound|outbound> endpoint=<ENDPOINT> encryption=ML-KEM+X25519 status=ready
event=connection_established peer_id=<PEER_ID> transport=QUIC/libp2p encryption_active=true
```

**What this means**: 
- ML-KEM (Kyber768) + X25519 handshake completed successfully
- Encrypted tunnel is active and ready for traffic
- Post-quantum cryptography is protecting the connection

### Key Rotation

Every 5 minutes (or `CRYPRQ_ROTATE_SECS`), you should see:

```
event=rotation_task_started interval_secs=300
event=key_rotation status=success epoch=<N> duration_ms=<MS> interval_secs=300
```

**What this means**:
- New ML-KEM keypair generated
- Old keys securely zeroized
- Encryption continues with new keys (epoch incremented)

### VPN Mode

When VPN mode is enabled:

```
🔒 VPN MODE ENABLED - System-wide routing mode
Creating TUN interface for packet forwarding...
✅ TUN interface cryprq0 configured with IP 10.0.0.1
VPN Mode: Listener will accept connections and route traffic through TUN interface
```

**What this means**: System-wide VPN routing is active (requires admin privileges).

### Packet Forwarding

When packets are forwarded (debug level, enable with `RUST_LOG=debug`):

```
🔐 ENCRYPT: Sent <N> bytes packet to <PEER_ID>
🔓 DECRYPT: Received <N> bytes packet from peer <PEER_ID>
🔐 Read <N> bytes from TUN, encrypting and forwarding
🔓 Received <N> bytes from tunnel, decrypting and writing to TUN
```

**What this means**: Packets are being encrypted, forwarded, and decrypted through the tunnel.

## Expected Log Sequence

### Healthy Listener Startup

1. `event=listener_starting` - Listener initializing
2. `event=listener_ready` - Listener accepting connections
3. `event=rotation_task_started` - Key rotation task started
4. `event=handshake_complete` - Connection established (when dialer connects)
5. `event=connection_established` - Encrypted tunnel active
6. `event=key_rotation status=success` - Periodic key rotation (every 5 min)

### Healthy Dialer Connection

1. `event=dialer_starting` - Dialer initializing
2. `event=handshake_complete` - Handshake completed
3. `event=connection_established` - Encrypted tunnel active
4. `event=key_rotation status=success` - Periodic key rotation (every 5 min)

## Troubleshooting

### No Handshake Logs

**Problem**: Connection established but no `event=handshake_complete` logs.

**Possible causes**:
- Log level too high (use `RUST_LOG=info`)
- Connection failed silently
- Peer denied (check for `event=peer_denied`)

**Solution**: Set `RUST_LOG=info,cryprq=debug` and check for connection errors.

### No Key Rotation Logs

**Problem**: No `event=key_rotation` logs after 5 minutes.

**Possible causes**:
- Key rotation task not started
- Log level too high
- Process exited before rotation

**Solution**: Verify `CRYPRQ_ROTATE_SECS` is set and check process is still running.

### VPN Mode Fails

**Problem**: `Failed to configure TUN interface IP` warning.

**Possible causes**:
- Missing admin privileges
- TUN interface already exists
- Platform not supported

**Solution**: Run with `sudo` (Linux/macOS) or check platform support.

## Log Filtering Examples

### Show Only Handshake Events

```bash
grep "event=handshake_complete" cryprq.log
```

### Show Only Key Rotation Events

```bash
grep "event=key_rotation" cryprq.log
```

### Show All Connection Events

```bash
grep "event=.*connection\|event=.*handshake" cryprq.log
```

### Show Errors and Warnings

```bash
grep -E "ERROR|WARN|error|warn" cryprq.log
```

## Security Notes

- **No sensitive data logged**: Keys, secrets, and plaintext packet contents are never logged
- **Peer IDs only**: Logs show peer IDs (public identifiers), not private keys
- **Epoch counters**: Key rotation logs show epoch numbers, not actual keys
- **Debug level**: Packet-level logs are debug-only and don't expose sensitive data

## Environment Variables

- `RUST_LOG`: Control log level (e.g., `info`, `debug`, `trace`)
- `CRYPRQ_ROTATE_SECS`: Key rotation interval in seconds (default: 300)

## Example Full Log Output

```
event=listener_starting peer_id=12D3KooW... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 transport=QUIC/libp2p
Local peer id: 12D3KooW...
event=listener_ready peer_id=12D3KooW... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 status=accepting_connections
Listening on /ip4/0.0.0.0/udp/9999/quic-v1
event=rotation_task_started interval_secs=300
event=handshake_complete peer_id=12D3KooW... direction=inbound endpoint=... encryption=ML-KEM+X25519 status=ready
event=connection_established peer_id=12D3KooW... transport=QUIC/libp2p encryption_active=true
event=key_rotation status=success epoch=1 duration_ms=15 interval_secs=300
```

This sequence confirms:
1. ✅ Listener started successfully
2. ✅ Connection established with encryption
3. ✅ Key rotation working correctly

