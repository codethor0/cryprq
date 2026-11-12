# CrypRQ VPN and P2P Tunnel Test Plan

## Overview

CrypRQ provides a QUIC-based, ML-KEM+X25519 encrypted P2P tunnel for secure peer-to-peer communication. The system includes:

- **P2P Tunnel Mode**: Encrypted tunnel between peers using QUIC transport and hybrid post-quantum cryptography (ML-KEM Kyber768 + X25519). This mode is fully functional in Docker/web environments.

- **System-Wide VPN Mode**: CLI support for system-wide routing via TUN interface. This mode requires root/admin privileges and TUN device support. In Docker/web environments, VPN mode is expected to fail gracefully with clear error messages, while P2P tunnel functionality remains available.

## P2P Tunnel E2E Tests (Automated)

The automated E2E test suite validates core tunnel functionality via `docker/e2e/docker-compose.e2e.yml`. All tests are executed in Docker containers to ensure consistent behavior.

### Test Suite Structure

The Docker harness covers the **legacy transport only** and runs exactly two tests:

1. **01_legacy_quic_connection**: two test-mode containers establish a libp2p/QUIC connection. This is not the CrypRQ handshake.
2. **02_legacy_rotation_timer**: the legacy local rotation timer fires at its interval. These are not the session keys of the authenticated protocol.

Three former tests of this harness (`02_mlkem_x25519_hybrid`, `04_file_transfer_integrity`, `05_packet_validation`) only searched the logs for words and are retired. The hybrid handshake, file integrity and packet validation are asserted by the authenticated two-process scripts and the Rust protocol tests; the mapping is in [E2E_COVERAGE_MAP.md](E2E_COVERAGE_MAP.md).

### Expected Results

Both tests must pass (exit code 0). The harness prints `E2E_RESULT passed=2 failed=0 skipped=0` and fails on any other count. The events it looks for:

- `event=handshake_complete` or `event=connection_established` in the logs of both containers. The line carries `encryption=libp2p-transport cryprq_handshake=none`: it reports the transport handshake, not a CrypRQ key agreement. (Until November 2026 it said `encryption=ML-KEM+X25519`, which was wrong for this path.)
- Three distinct `event=key_rotation status=success epoch=N` lines at the configured interval.

### Running the E2E Suite

From repository root:

```bash
docker compose -f docker/e2e/docker-compose.e2e.yml up --build -d listener dialer
bash docker/e2e/test_suite.sh
docker compose -f docker/e2e/docker-compose.e2e.yml down -v --remove-orphans
```

Expected outcome: `E2E_RESULT passed=2 failed=0 skipped=0`, exit code 0. The compose file publishes UDP port 9999 on the host; the tests themselves only use `docker exec` and `docker logs`.

The protocol itself is tested with:

```bash
cargo build --release -p cryprq --locked
bash scripts/e2e-authenticated-vpn.sh ./target/release/cryprq
bash scripts/e2e-authenticated-file-transfer.sh ./target/release/cryprq
```

## Web UI + P2P Manual Tests

The web UI provides a browser-based interface for testing P2P tunnel functionality without CLI access.

### Two-Tab Listener/Dialer Test

**Setup:**
1. Start web stack: `docker compose -f docker-compose.web.yml up --build -d`
2. Open http://localhost:8787 in two browser tabs

**Test Steps:**
1. Tab 1: Set mode to "Listener", configure port (default 10000), click "Connect"
2. Verify: UI shows listening state, debug console shows `listener_ready` events
3. Tab 2: Set mode to "Dialer", configure peer address (e.g., `/ip4/127.0.0.1/udp/10000/quic-v1`), click "Connect"
4. Verify: Both tabs show connected/encrypted state, debug console shows `handshake_complete` and `connection_established` events

**Expected Behavior:**
- Connection status indicators show "Encrypted tunnel active"
- Encryption Status panel displays ML-KEM+X25519 hybrid encryption
- Connection Health panel shows handshake direction, ping latency, current epoch
- No console errors related to mode switching or connection establishment

### File Transfer Test (if implemented in UI)

**Test Steps:**
1. Establish Listener/Dialer connection (from above)
2. On sender side: Use "Send File Securely" control, select a test file
3. Verify: Progress indicators update, file transfer completes
4. On receiver side: Verify file appears, download works
5. Integrity check: Compare SHA-256 hash of original and received files

**Expected Behavior:**
- File transfer completes without errors
- SHA-256 hashes match exactly
- Debug console shows file transfer events
- UI shows transfer progress and completion status

## VPN Mode Error-Handling Tests (Current Environment)

In Docker/web environments without root privileges or TUN device support, VPN mode is expected to fail gracefully. These tests validate that error handling is clear and non-destructive.

### Test: Listener with --vpn

**Command:**
```bash
docker exec cryprq-web /usr/local/bin/cryprq --listen /ip4/0.0.0.0/udp/10000/quic-v1 --vpn
```

**Expected Outcome:**
- Process prints: `VPN MODE ENABLED - System-wide routing mode`
- Process prints: `Creating TUN interface for packet forwarding...`
- Process prints: `Creating TUN interface cryprq0 for VPN mode`
- Process exits with error: `Failed to create TUN interface`
- Error message includes: `requires root/admin privileges` or `No such file or directory`
- Exit code: 1 (non-zero, but no panic or crash)

**Classification:**
- PASS (VPN mode error-handling): Clear error messages, graceful failure
- ENV LIMITATION: System-wide VPN cannot be fully tested without root/TUN privileges

### Test: Dialer with --vpn

**Command:**
```bash
docker exec cryprq-web /usr/local/bin/cryprq --peer /ip4/127.0.0.1/udp/10000/quic-v1 --vpn
```

**Expected Outcome:**
- Same error sequence as Listener test above
- Exit code: 1
- No crash or hang

**Classification:**
- PASS (VPN mode error-handling): Clear error messages, graceful failure
- ENV LIMITATION: System-wide VPN cannot be fully tested without root/TUN privileges

### Validation Criteria

VPN mode error-handling is considered correct when:
- Error messages explicitly state TUN/privilege requirements
- Process exits cleanly (non-zero exit code, no panic)
- P2P tunnel mode remains available (non-VPN mode works correctly)
- UI surfaces VPN errors clearly without hiding P2P functionality

## Future OS-Level VPN Validation (Out-of-Scope Here)

Full system-wide VPN testing requires OS-level integration and cannot be performed in Docker containers without privileged access. The following tests are future work:

### System-Wide Routing Verification

**Requirements:**
- Root/admin privileges
- Real TUN device (`/dev/net/tun` on Linux, Network Extension on macOS)
- OS-level routing configuration access

**Tests:**
- Confirm default route goes via TUN interface (e.g., `cryprq0`) when VPN is active
- Verify route reverts when VPN is stopped
- Test route persistence across VPN restart

### Traffic Leak Tests

**Requirements:**
- System-wide VPN client with proper kernel/TUN integration
- External connectivity for leak detection services

**Tests:**
- **IP Leak**: Verify external IP check shows VPN endpoint IP, not local IP
- **DNS Leak**: Confirm all DNS queries go through tunnel/remote resolver
- **WebRTC Leak**: Test WebRTC IP disclosure prevention
- **Kill-Switch**: Verify traffic is blocked when tunnel drops unexpectedly

### macOS Network Extension Integration

**Requirements:**
- macOS with Network Extension framework
- NEPacketTunnelProvider implementation
- System Extension installation and approval

**Tests:**
- Install Network Extension via System Preferences
- Start/stop VPN from system UI
- Verify packet flow through NEPacketTunnelProvider
- Test uninstall and cleanup

### Multi-Peer / Multi-Session Stress Tests

**Requirements:**
- Multiple concurrent tunnel sessions
- Resource monitoring capabilities

**Tests:**
- Many concurrent dialers connecting to single listener
- Rapid connect/disconnect loops
- Memory and CPU usage under load
- Connection stability during key rotation events

## Test Environment Notes

### Current Environment (Docker/Web)

- **P2P Tunnel**: Fully validated via E2E suite and web UI
- **VPN Mode**: Error-handling validated; full VPN functionality requires OS-level testing
- **Limitations**: TUN device creation, root privileges, system routing configuration

### Future Environment (OS-Level)

- **Requirements**: Root/admin access, TUN device, OS network stack integration
- **Scope**: System-wide routing, leak prevention, kill-switch behavior
- **Tools**: External IP check services, DNS leak test services, network monitoring tools

## Host-Mode Tests

For testing CrypRQ's VPN mode on real host machines (macOS/Linux with TUN + root privileges), see:

- **`docs/VPN_HOST_TEST_PLAN.md`**: Detailed host-mode test plan including IP/DNS/WebRTC behavior checks, performance testing, and failure-mode observations
- **`scripts/vpn_host_probe.sh`**: Quick probe script for automated host-mode validation

The host-mode test plan covers:
- Basic tunnel sanity tests (local loopback, two-host)
- IP/DNS/WebRTC behavior checks
- Performance and stability tests (throughput, latency, long-lived connections)
- Failure-mode observations (process termination, network disconnection, routing behavior)

**Note:** Host-mode tests require root/admin privileges and TUN support. If these are not available, tests will be classified as ENV LIMITATION.

## Leak & Routing Checks (Host-Mode)

For comprehensive leak and routing diagnostics when using host-mode VPN, see:

- **`docs/VPN_LEAK_AND_ROUTING_TEST_PLAN.md`**: Detailed procedures for IP, DNS, IPv6, and routing leak detection
- **`scripts/vpn_leak_probe.sh`**: Automated probe script for collecting leak diagnostic information

The leak probe script is **informational only** and does not replace manual verification. It collects:

- IP address observations (IPv4 and IPv6)
- DNS resolver configuration
- Routing table and interface information
- Detection of CrypRQ VPN interface presence

Run `scripts/vpn_leak_probe.sh` before and after starting VPN to observe changes. Interpret results using `docs/VPN_LEAK_AND_ROUTING_TEST_PLAN.md` and your deployment architecture.

## References

- `docs/VPN_HOST_TEST_PLAN.md`: Host-mode VPN testing procedures (macOS/Linux)
- `docs/VPN_LEAK_AND_ROUTING_TEST_PLAN.md`: VPN leak & routing test plan
- `docker/e2e/docker-compose.e2e.yml`: E2E test harness configuration
- `docker-compose.web.yml`: Web stack configuration
- `docs/VALIDATION_RUN_RELEASE_12f7bd6.md`: Release validation results
- `docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`: Web stack release notes

