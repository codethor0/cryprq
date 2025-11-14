# CrypRQ Operator Logs Guide

## Overview

CrypRQ uses structured logging to provide observability into the encrypted tunnel's behavior. All logs follow a consistent format with `event=` prefixes for easy parsing and filtering.

## Log Levels

- `RUST_LOG=info` - Recommended for production (shows key events)
- `RUST_LOG=debug` - Detailed debugging (includes packet-level details)
- `RUST_LOG=trace` - Maximum verbosity (not recommended for production)

## Structured Events

### Connection Lifecycle

```
event=listener_starting
```
Emitted when the listener begins binding to its address.

```
event=listener_ready
```
Emitted when the listener is ready to accept connections.

```
event=dialer_starting
```
Emitted when the dialer begins connecting to a peer.

```
event=handshake_complete
```
Emitted when the ML-KEM + X25519 handshake completes successfully.

```
event=connection_established
```
Emitted when the encrypted tunnel is fully established and ready for data transfer.

### Key Rotation

```
event=rotation_task_started
```
Emitted when the key rotation background task starts.

```
event=key_rotation
```
Emitted every time keys are rotated (default: every 300 seconds).

### VPN Mode

```
event=tun_interface_created
```
Emitted when the TUN interface is created (VPN mode).

```
event=packet_forwarding_started
```
Emitted when packet forwarding begins through the encrypted tunnel.

## Log Categories

### Peer Events
- `event=listener_starting`
- `event=listener_ready`
- `event=dialer_starting`
- `event=handshake_complete`
- `event=connection_established`

### Rotation Events
- `event=rotation_task_started`
- `event=key_rotation`

### Status Events
- `event=tun_interface_created`
- `event=packet_forwarding_started`

### File Transfer Events
- `Receiving file: <filename> (<size> bytes) from peer <PEER_ID>`
- `Sent file metadata to <PEER_ID>: <filename> (<size> bytes)`
- `Received chunk <N> from peer <PEER_ID> (<size> bytes)` (debug level)
- `End packet received from peer <PEER_ID>` (debug level)
- `File received successfully: <filename> (<size> bytes) from peer <PEER_ID>`
- `File transfer task completed - all packets sent`
- `All responses received (<N>/<N>) - exiting`

### Error Events
- Any log line containing `ERROR` or `error=`
- `Request failed for <request_id>: <error>` (file transfer failures)
- `File transfer callback failed for peer <PEER_ID>: <error>`

## Example Log Output

```
INFO  cryprq::p2p event=listener_starting addr=/ip4/0.0.0.0/udp/9999/quic-v1
INFO  cryprq::p2p event=listener_ready
INFO  cryprq::p2p event=dialer_starting peer=/ip4/127.0.0.1/udp/9999/quic-v1
INFO  cryprq::p2p event=handshake_complete peer_id=12D3KooW...
INFO  cryprq::p2p event=connection_established peer_id=12D3KooW...
INFO  cryprq::p2p event=rotation_task_started interval_secs=300
INFO  cryprq::p2p event=key_rotation
```

## Filtering Logs

### Using grep

```bash
# Show only handshake events
RUST_LOG=info ./target/release/cryprq --listen ... | grep "event=handshake"

# Show only rotation events
RUST_LOG=info ./target/release/cryprq --listen ... | grep "event=rotation"

# Show errors
RUST_LOG=info ./target/release/cryprq --listen ... | grep -i error
```

### Using jq (if logs are JSON)

If you configure JSON logging, you can use `jq`:

```bash
RUST_LOG=info ./target/release/cryprq --listen ... | jq 'select(.event=="handshake_complete")'
```

## Web UI Integration

The web UI (`web/server/server.mjs`) parses these structured logs and categorizes them for display:

- **Peer Events**: Connection lifecycle events
- **Rotation Events**: Key rotation events
- **Status Events**: System status updates
- **Error Events**: Error conditions

## File Transfer Log Interpretation

### Successful File Transfer Flow

**Sender Side:**
1. Connection established: `Connected to peer: <PEER_ID>`
2. Expected responses calculated: `Expected <N> responses (1 metadata + <chunks> chunks + 1 end)`
3. Metadata sent: `Sent file metadata to <PEER_ID>: <filename> (<size> bytes)`
4. Responses received: `Received response for request OutboundRequestId(<N>): 2 bytes`
5. Completion: `All responses received (<N>/<N>) - exiting`

**Receiver Side:**
1. File reception started: `Receiving file: <filename> (<size> bytes) from peer <PEER_ID>`
2. Chunks received: `Received chunk <N> from peer <PEER_ID> (<size> bytes)` (debug level)
3. End packet received: `End packet received from peer <PEER_ID>` (debug level)
4. File written: `File received successfully: <filename> (<size> bytes) from peer <PEER_ID>`

### File Transfer Failure Indicators

**Protocol Negotiation Issues:**
- No `Codec::write_request` or `Codec::read_request` logs (protocol negotiation failed)
- `Request failed for <request_id>: ProtocolNegotiationFailed` or similar
- `Timeout waiting for responses (<received>/<expected>) - exiting anyway`

**Common Issues:**
- **No responses received**: Protocol negotiation may not have completed. Check that event loop is running.
- **Partial transfer**: Some chunks missing. Check network stability and connection logs.
- **Hash mismatch**: `File hash mismatch` error indicates data corruption. Retry transfer.
- **Callback not triggered**: Receiver callback not registered. Check receiver startup logs.

### File Transfer Troubleshooting

**Sender shows "Timeout waiting for responses":**
- Verify receiver is running and listening on correct address
- Check that peer ID in multiaddr matches receiver's peer ID
- Ensure connection is established before sending file
- Check for protocol negotiation errors in logs

**Receiver shows "No file transfer callback registered":**
- Verify receiver was started with `receive-file` command
- Check that output directory exists and is writable
- Review receiver startup logs for callback registration

**File received but hash mismatch:**
- Network corruption or incomplete transfer
- Verify all chunks were received (check chunk logs)
- Retry file transfer
- Check for network instability or packet loss

## Troubleshooting

### No handshake events
- Check that both peers can reach each other (network connectivity)
- Verify firewall rules allow UDP traffic
- Check that addresses are correct (multiaddr format)

### No rotation events
- Verify `CRYPRQ_ROTATE_SECS` environment variable is set
- Check that the rotation task started (`event=rotation_task_started`)

### Connection fails
- Look for error events in logs
- Verify both peers are using compatible versions
- Check network connectivity and firewall rules

### File transfer fails
- Verify connection is established before sending file
- Check that peer ID in sender command matches receiver's peer ID
- Ensure receiver callback is registered (check receiver startup logs)
- Review protocol negotiation logs for errors
- Check network stability and firewall rules

