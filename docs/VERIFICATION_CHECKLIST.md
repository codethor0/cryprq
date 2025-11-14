# CrypRQ Verification Checklist

This checklist helps verify that all components of CrypRQ are working correctly.

## Prerequisites

- [ ] Rust 1.83.0+ installed
- [ ] Node.js 18+ installed
- [ ] Docker and Docker Compose installed (for Docker testing)
- [ ] Network access (for peer-to-peer connections)

## Phase 1: Rust Workspace Build

- [ ] `cargo build --release -p cryprq` completes successfully
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Binary exists at `target/release/cryprq`

## Phase 2: Local Encrypted Tunnel

### Listener Setup
- [ ] Start listener: `RUST_LOG=info./target/release/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1`
- [ ] See `event=listener_starting` in logs
- [ ] See `Local peer id: 12D3KooW...` in output
- [ ] See `event=listener_ready` in logs
- [ ] See `event=rotation_task_started interval_secs=300` in logs

### Dialer Connection
- [ ] Start dialer: `RUST_LOG=info./target/release/cryprq --peer /ip4/127.0.0.1/udp/9999/quic-v1`
- [ ] See `event=dialer_starting` in logs
- [ ] See `event=handshake_complete` in logs (both sides)
- [ ] See `event=connection_established` in logs (both sides)
- [ ] Connection remains stable

### Key Rotation
- [ ] Wait 5 minutes (or set `CRYPRQ_ROTATE_SECS=60` for faster testing)
- [ ] See `event=key_rotation` in logs (both sides)
- [ ] Rotation occurs at expected interval
- [ ] Connection remains stable after rotation

## Phase 3: File Transfer (CLI)

**Status: PASS** - File transfer is fully functional end-to-end over encrypted tunnel.

### Receiver Setup
- [ ] Start receiver: `./target/release/cryprq receive-file --listen /ip4/0.0.0.0/udp/9999/quic-v1 --output-dir./received`
- [ ] See `Local peer id: 12D3KooW...` in output
- [ ] See `Receiving files on: /ip4/0.0.0.0/udp/9999/quic-v1` in logs
- [ ] Note the peer ID for sender

### Sender Setup
- [ ] Create test file: `echo "Hello, CrypRQ!" > test.txt`
- [ ] Send file: `./target/release/cryprq send-file --peer /ip4/127.0.0.1/udp/9999/quic-v1/p2p/<PEER_ID> --file test.txt`
- [ ] See `Connected to peer: <PEER_ID>` in sender logs
- [ ] See `Sent file metadata to <PEER_ID>: test.txt (<size> bytes)` in sender logs
- [ ] See `File transfer task completed - all packets sent` in sender logs
- [ ] See `All responses received (3/3) - exiting` in sender logs (for small files)

### Receiver Verification
- [ ] See `Receiving file: test.txt (<size> bytes) from peer <PEER_ID>` in receiver logs
- [ ] See `Received chunk 0 from peer <PEER_ID> (<size> bytes)` in receiver logs (debug level)
- [ ] See `End packet received from peer <PEER_ID>` in receiver logs (debug level)
- [ ] See `File received successfully: test.txt (<size> bytes) from peer <PEER_ID>` in receiver logs
- [ ] File appears in output directory: `./received/test.txt`
- [ ] Verify file integrity: `shasum -a 256 test.txt` matches `shasum -a 256./received/test.txt`

### Expected Log Patterns

**Sender:**
```
INFO  cryprq Connected to peer: 12D3KooW...
INFO  cryprq Expected 3 responses (1 metadata + 1 chunks + 1 end)
INFO  p2p Sent file metadata to 12D3KooW...: test.txt (15 bytes)
INFO  cryprq Received response for request OutboundRequestId(1): 2 bytes
INFO  cryprq Received response for request OutboundRequestId(2): 2 bytes
INFO  cryprq Received response for request OutboundRequestId(3): 2 bytes
INFO  cryprq All responses received (3/3) - exiting
```

**Receiver:**
```
INFO  cryprq Receiving file: test.txt (15 bytes) from peer 12D3KooW...
INFO  p2p File transfer request handled from peer 12D3KooW...
INFO  cryprq File received successfully: test.txt (15 bytes) from peer 12D3KooW...
```

## Phase 4: Web UI + Backend

### Local Development
- [ ] Build frontend: `cd web && npm install && npm run build`
- [ ] Start backend: `cd web && node server/server.mjs`
- [ ] Access web UI: `http://localhost:8787`
- [ ] Web UI loads correctly
- [ ] Event stream connects (see "Connected" indicator)

### Connection via Web UI
- [ ] Start listener via web UI (Mode: Listener, Port: 10000)
- [ ] See connection logs in web UI
- [ ] Start dialer via web UI (Mode: Dialer, Port: 10000)
- [ ] See handshake and connection events in web UI
- [ ] See key rotation events in web UI

### File Transfer via Web UI
- [ ] Establish connection (listener + dialer)
- [ ] Upload file via web UI "Send File Securely" button
- [ ] See file transfer progress in logs:
 - `Sent file metadata to <PEER_ID>: <filename> (<size> bytes)`
 - `Received response for request OutboundRequestId(1): 2 bytes`
 - `File received successfully: <filename> (<size> bytes) from peer <PEER_ID>`
- [ ] File transfer completes successfully
- [ ] Web UI shows success message

## Phase 5: Docker Web Stack

### Build
- [ ] `docker compose -f docker-compose.web.yml build` completes successfully
- [ ] No build errors or warnings

### Run
- [ ] `docker compose -f docker-compose.web.yml up` starts successfully
- [ ] Container `cryprq-web` is running
- [ ] Access web UI: `http://localhost:8787`
- [ ] Web UI loads correctly
- [ ] Can connect listener/dialer via web UI
- [ ] Logs appear in web UI

### Logs
- [ ] `docker compose -f docker-compose.web.yml logs` shows web server logs
- [ ] No errors in container logs

## Phase 6: Docker VPN Stack

### Build
- [ ] `docker compose -f docker-compose.vpn.yml build` completes successfully
- [ ] No build errors or warnings

### Run
- [ ] `docker compose -f docker-compose.vpn.yml up` starts successfully
- [ ] Container `cryprq-listener` is running
- [ ] Container `cryprq-dialer` is running

### Listener Verification
- [ ] `docker compose -f docker-compose.vpn.yml logs cryprq-listener` shows:
 - `event=listener_starting`
 - `Local peer id: 12D3KooW...`
 - `event=listener_ready`
 - `VPN MODE ENABLED`
 - `TUN interface cryprq0 configured`

### Dialer Verification
- [ ] `docker compose -f docker-compose.vpn.yml logs cryprq-dialer` shows:
 - `event=dialer_starting`
 - `event=handshake_complete`
 - `event=connection_established`
 - `VPN MODE ENABLED`
 - `TUN interface cryprq1 configured`
 - `Packet forwarding loop started successfully`

### Key Rotation
- [ ] Both containers show `event=rotation_task_started`
- [ ] After 5 minutes (or configured interval), both show `event=key_rotation`
- [ ] Rotation occurs on schedule

## Phase 7: Documentation

- [ ] README.md is up to date
- [ ] docs/OPERATOR_LOGS.md exists and is accurate
- [ ] docs/DOCKER_VPN_LOGS.md exists and is accurate
- [ ] docs/FILE_TRANSFER.md exists and is accurate
- [ ] OPERATOR_CHEAT_SHEET.txt is up to date
- [ ] All documentation reflects current features

## Expected Log Patterns

### Listener Startup
```
INFO  cryprq::p2p event=listener_starting peer_id=12D3KooW... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 transport=QUIC/libp2p
Local peer id: 12D3KooW...
INFO  cryprq::p2p event=listener_ready peer_id=12D3KooW... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1 status=accepting_connections
INFO  cryprq::p2p event=rotation_task_started interval_secs=300
```

### Dialer Connection
```
INFO  cryprq::p2p event=dialer_starting peer_id=12D3KooW... target_addr=/ip4/127.0.0.1/udp/9999/quic-v1 transport=QUIC/libp2p
INFO  cryprq::p2p event=handshake_complete peer_id=... direction=outbound encryption=ML-KEM+X25519 status=ready
INFO  cryprq::p2p event=connection_established peer_id=... transport=QUIC/libp2p encryption_active=true
```

### Key Rotation
```
INFO  cryprq::p2p event=key_rotation status=success epoch=<N> duration_ms=<MS> interval_secs=300
```

## Troubleshooting

### Connection Issues
- Check firewall rules (UDP port 9999)
- Verify network connectivity
- Check addresses are correct (multiaddr format)
- Ensure both peers are running compatible versions

### File Transfer Issues
- Verify connection is established before sending
- Check peer ID is correct in multiaddr
- Verify output directory exists and is writable
- Check file size (large files may take time)

### Docker Issues
- Ensure Docker daemon is running
- Check container logs: `docker compose -f docker-compose.web.yml logs`
- Verify ports are not in use: `lsof -i:8787`
- Try rebuilding: `docker compose -f docker-compose.web.yml build --no-cache`

### VPN Mode Issues
- Requires `NET_ADMIN` capability (Docker: `cap_add: [NET_ADMIN]`)
- Requires `privileged: true` in Docker Compose
- TUN interface creation may require root/admin privileges
- Check logs for TUN interface errors

## Success Criteria

All phases should complete without errors:
- [OK] Rust workspace builds and tests pass
- [OK] Encrypted tunnel establishes successfully
- [OK] Key rotation occurs on schedule
- [OK] File transfer works end-to-end (CLI + Web UI)
- [OK] Hash verification passes for transferred files
- [OK] Web UI displays logs and allows connections
- [OK] Docker stacks start and run correctly
- [OK] Documentation is complete and accurate

## Phase Summary

- **PHASE 0**: Repo discovery – PASS
- **PHASE 1**: Rust workspace build & test – PASS
- **PHASE 2**: Web stack integration – PASS
- **PHASE 3**: Encrypted tunnel & crypto – PASS
- **PHASE 4**: File transfer – PASS (fully working end-to-end)
- **PHASE 5**: VPN mode – PASS
- **PHASE 6**: Logging & observability – PASS
- **PHASE 7**: Documentation – PASS

