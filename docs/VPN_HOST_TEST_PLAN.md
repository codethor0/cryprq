# CrypRQ VPN Host-Mode Test Plan (macOS / Linux)

## Overview

This document provides a test plan for validating CrypRQ's VPN mode (`--vpn` flag) when running on real host machines (macOS or Linux) with TUN/TAP support and root/admin privileges. This plan complements `docs/VPN_TEST_PLAN.md`, which focuses on Docker-based P2P tunnel testing and error handling.

**Related Documentation:**
- `docs/VPN_TEST_PLAN.md`: Docker-based P2P tunnel and VPN error handling tests
- `docs/ARCHITECTURE_AND_THREAT_MODEL.md`: Architecture overview and threat model
- `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md`: Troubleshooting guide and support bundle collection
- `scripts/vpn_host_probe.sh`: Automated probe script for host-mode validation

## Scope & Assumptions

### Scope

This test plan is designed for:
- Testing CrypRQ when used in `--vpn` mode on a real host (not Docker containers)
- Validating system-wide routing behavior when TUN interfaces are created
- Checking for basic network-layer issues (IP/DNS/WebRTC behavior)
- Measuring performance characteristics (throughput, latency)
- Observing failure-mode behavior (process termination, network disconnection)

### Key Assumptions

As documented in `docs/ARCHITECTURE_AND_THREAT_MODEL.md`:

**CrypRQ's Focus:**
- CrypRQ focuses on **confidentiality and integrity of data between peers**, not anonymity
- System-wide routing requires OS-level integration and correct host configuration
- VPN mode is a feature for system-wide routing, not a consumer VPN replacement

**Non-Goals:**
- **No anonymity guarantees:** CrypRQ does not hide the fact that two endpoints are communicating
- **No global passive adversary resistance:** Traffic patterns and metadata are not obfuscated
- **No guaranteed OS-level kill-switch:** CrypRQ does not currently implement a formal kill-switch mechanism

### Test Classification

All tests should be classified using the same system as `docs/CI_E2E_INVESTIGATION_RUNBOOK.md`:

- **PASS:** Test succeeded as expected
- **FAIL (Product Bug):** Test revealed an actual issue in CrypRQ's behavior
- **ENV LIMITATION:** Test could not be run due to environment constraints (missing TUN, no root, TLS CA issues, missing tools, etc.)

## Prerequisites

### Supported Platforms

- **macOS:** Requires Network Extension framework or equivalent TUN support
- **Linux:** Requires `/dev/net/tun` device and TUN/TAP kernel support

### Requirements

**Required:**
- `cryprq` binary installed on host and available in PATH (or via `CRYPRQ_BIN` environment variable)
- Root/admin privileges to create TUN/TAP interfaces and configure routes
- TUN/TAP support in kernel:
  - Linux: `/dev/net/tun` device must exist
  - macOS: Network Extension framework or equivalent TUN support

**Recommended (Optional, Not Hard Dependencies):**
- `curl`: For IP address checks via public IP services
- `dig` or `nslookup`: For DNS resolver checks
- `iperf3`: For throughput and latency testing
- Modern web browser: For WebRTC leak tests via external test sites
- `ping`: For basic connectivity tests

**Note:** If recommended tools are missing, tests that depend on them should be classified as **ENV LIMITATION**, not **FAIL**.

## Basic Tunnel Sanity Tests

### Local Loopback VPN Test

**Goal:** Verify that CrypRQ can create a TUN interface and establish a VPN tunnel on a single host.

**Setup:**
1. Ensure you have root/admin privileges
2. Verify TUN support is available (see Prerequisites)
3. Choose a test port (e.g., 10000)

**Steps:**
1. **Start Listener with VPN:**
   ```bash
   sudo cryprq --listen /ip4/0.0.0.0/udp/10000/quic-v1 --vpn
   ```
   - Observe logs for:
     - "VPN MODE ENABLED - System-wide routing mode"
     - "Creating TUN interface for packet forwarding..."
     - Successful TUN creation (no "Failed to create TUN device" errors)
     - Listener ready message

2. **Start Dialer with VPN (in separate terminal):**
   ```bash
   sudo cryprq --peer /ip4/127.0.0.1/udp/10000/quic-v1 --vpn
   ```
   - Observe logs for:
     - "VPN MODE ENABLED - System-wide routing mode"
     - "Creating TUN interface cryprq0 for VPN mode"
     - Successful TUN creation
     - Handshake completion
     - Connection established

3. **Verify Tunnel Connectivity:**
   - Use `ping` or `iperf3` to test connectivity between TUN interfaces
   - Verify that traffic flows through the tunnel

**Classification:**
- **PASS:** Both listener and dialer start successfully, TUN interfaces are created, and basic connectivity works
- **FAIL (Product Bug):** Processes crash unexpectedly, TUN creation fails with unclear errors, or connectivity fails despite successful startup
- **ENV LIMITATION:** TUN creation fails with clear "requires root/admin privileges" or "No such file or directory" errors, or missing TUN support

### Two-Host VPN Test

**Goal:** Verify VPN tunnel between two separate hosts.

**Setup:**
1. Two hosts on the same network (or reachable via public IPs)
2. Both hosts have `cryprq` installed and root/admin privileges
3. Firewall rules allow UDP traffic on chosen port

**Steps:**
1. **Host A (Listener):**
   ```bash
   sudo cryprq --listen /ip4/0.0.0.0/udp/10000/quic-v1 --vpn
   ```
   - Note the public IP or hostname of Host A

2. **Host B (Dialer):**
   ```bash
   sudo cryprq --peer /ip4/<HOST_A_IP>/udp/10000/quic-v1 --vpn
   ```
   - Replace `<HOST_A_IP>` with Host A's IP address

3. **Verify Connectivity:**
   - Test connectivity between TUN interfaces on both hosts
   - Verify routing tables show routes through TUN interfaces

**Classification:**
- **PASS:** Tunnel establishes successfully between hosts, routing works correctly
- **FAIL (Product Bug):** Handshake fails, routing misconfiguration, or connectivity issues
- **ENV LIMITATION:** Network connectivity issues, firewall blocking, or missing TUN support on either host

## IP / DNS / WebRTC Behavior Checks

### Understanding These Checks

These checks validate **behavior** when CrypRQ is configured as a system VPN (default route). They are **not proofs of anonymity** but rather confirm that routing is working as configured.

**Important:** CrypRQ does not provide anonymity guarantees. These tests are for validating routing behavior, not privacy properties.

### IP Address Check

**Goal:** Verify that external IP address changes (or remains consistent) based on VPN routing configuration.

**Before Starting VPN:**
1. Note your external IP address using a public IP service:
   ```bash
   curl -s https://api.ipify.org
   # or
   curl -s https://ifconfig.me
   ```
   - Record this as "baseline IP"

**After Starting VPN:**
1. With CrypRQ VPN active (listener and dialer both running with `--vpn`):
   ```bash
   curl -s https://api.ipify.org
   ```
   - Record this as "VPN IP"

**Interpretation:**
- If CrypRQ is configured to route traffic through the peer, expect the VPN IP to differ from baseline IP
- If CrypRQ is configured for local P2P only, expect the VPN IP to match baseline IP
- Behavior depends on routing configuration and deployment model

**Classification:**
- **PASS:** IP behavior matches expected routing configuration
- **INVESTIGATE:** IP behavior is inconsistent with configuration (may indicate routing issue)
- **ENV LIMITATION:** IP check fails due to TLS/CA errors, network connectivity, or missing `curl`

### DNS Leak Check

**Goal:** Verify which DNS resolvers are being used when VPN is active.

**Method 1: Using dig:**
```bash
dig @1.1.1.1 example.com
# Check which resolver actually responded
```

**Method 2: Using DNS leak test sites:**
- Visit a DNS leak test website (e.g., dnsleaktest.com) while VPN is active
- Record which DNS resolvers appear in the test results

**Interpretation:**
- If CrypRQ is routing DNS through the tunnel, expect to see tunnel-related resolvers
- If DNS is not routed through tunnel, expect to see local/ISP resolvers
- Behavior depends on routing configuration

**Classification:**
- **PASS:** DNS resolver behavior matches expected routing configuration
- **INVESTIGATE:** DNS resolver behavior is inconsistent with configuration
- **ENV LIMITATION:** DNS check fails due to missing `dig`, network connectivity, or TLS errors

### WebRTC Leak Check

**Goal:** Verify whether local/private IP addresses are exposed via WebRTC when VPN is active.

**Steps:**
1. Open a modern web browser
2. Visit a WebRTC leak test page (e.g., browserleaks.com/webrtc)
3. Record which IP addresses are detected:
   - Local/private IPs (e.g., 192.168.x.x, 10.x.x.x)
   - Public IPs

**Interpretation:**
- WebRTC may expose local IPs even when VPN is active (this is browser behavior, not necessarily a VPN issue)
- If public IP changes when VPN is active, routing may be working
- WebRTC behavior depends on browser and OS configuration

**Classification:**
- **PASS:** WebRTC behavior is consistent with expected routing (or matches baseline behavior)
- **INVESTIGATE:** WebRTC reveals unexpected IPs that suggest routing misconfiguration
- **ENV LIMITATION:** WebRTC test fails due to missing browser, network connectivity, or test site unavailability

## Performance & Stability Tests

### Throughput Testing

**Goal:** Measure data transfer throughput over the VPN tunnel.

**Baseline (No VPN):**
```bash
# On Host A (server)
iperf3 -s

# On Host B (client)
iperf3 -c <HOST_A_IP> -t 30
```

**With VPN:**
1. Start CrypRQ VPN on both hosts (listener and dialer with `--vpn`)
2. Run same `iperf3` test through TUN interfaces
3. Compare throughput: baseline vs VPN

**Expected:**
- VPN throughput will be lower than baseline due to encryption overhead
- Throughput should be reasonable (exact numbers depend on hardware and network)

**Classification:**
- **PASS:** Throughput is reasonable (e.g., >10 Mbps for typical hardware)
- **FAIL (Product Bug):** Throughput is extremely low (<1 Mbps) or connection drops
- **ENV LIMITATION:** `iperf3` not available, network connectivity issues, or missing TUN support

### Latency Testing

**Goal:** Measure round-trip latency over the VPN tunnel.

**Baseline (No VPN):**
```bash
ping -c 10 <PEER_IP>
```

**With VPN:**
1. Start CrypRQ VPN on both hosts
2. Run `ping` through TUN interfaces
3. Compare latency: baseline vs VPN

**Expected:**
- VPN latency will be slightly higher than baseline due to encryption/decryption
- Latency should be reasonable (e.g., <50ms additional latency for local network)

**Classification:**
- **PASS:** Latency is reasonable (e.g., <100ms additional latency)
- **FAIL (Product Bug):** Latency is extremely high (>500ms additional) or connection is unstable
- **ENV LIMITATION:** Network connectivity issues or missing TUN support

### Long-Lived Connection Test

**Goal:** Verify tunnel stability during extended sessions with key rotations.

**Steps:**
1. Start CrypRQ VPN on both hosts
2. Run a long `iperf3` test (e.g., 10 minutes) or continuous `ping`
3. Observe logs for:
   - Key rotation events occurring at expected intervals (default: 300 seconds)
   - No disconnects or stalls during rotation
   - Stable throughput/latency throughout the session

**Classification:**
- **PASS:** Tunnel remains stable through multiple key rotations, no disconnects
- **FAIL (Product Bug):** Tunnel disconnects, stalls, or throughput degrades significantly during rotation
- **ENV LIMITATION:** Test cannot be completed due to network issues or missing tools

## Failure-Mode / Kill-Switch-Adjacent Tests

### Important Clarification

**CrypRQ does not currently implement a formal kill-switch mechanism.** These tests are **behavioral observations** to understand how the system behaves when failures occur. They are **not** guarantees of safety properties.

### Process Termination Test

**Goal:** Observe what happens to network traffic when CrypRQ process is terminated.

**Steps:**
1. Start CrypRQ VPN on both hosts
2. Verify tunnel is active and routing traffic
3. Terminate one of the CrypRQ processes (e.g., `kill -9 <PID>`)
4. Observe:
   - What happens to active connections
   - Whether routing tables are cleaned up
   - Whether TUN interfaces are removed
   - Whether traffic falls back to normal routing or is blocked

**Classification:**
- **PASS:** Process terminates cleanly, TUN interfaces are removed, routing is restored (or behavior is documented)
- **FAIL (Product Bug):** Process leaves TUN interfaces or routes in place, causing network issues
- **ENV LIMITATION:** Cannot test due to missing TUN support or network configuration issues

### Network Interface Disconnection Test

**Goal:** Observe behavior when underlying network interface is disabled while VPN is active.

**Steps:**
1. Start CrypRQ VPN on both hosts
2. Verify tunnel is active
3. Disable the underlying network interface (e.g., `ifconfig eth0 down` on Linux, or disconnect Wi-Fi)
4. Observe:
   - How CrypRQ handles the disconnection
   - Whether it attempts to reconnect
   - Whether it exits gracefully or hangs

**Classification:**
- **PASS:** CrypRQ handles disconnection gracefully (exits, logs error, or attempts reconnection)
- **FAIL (Product Bug):** CrypRQ hangs, crashes, or leaves system in inconsistent state
- **ENV LIMITATION:** Cannot test due to missing TUN support or network configuration

### Routing Behavior After Failure

**Goal:** Verify that routing tables are restored correctly after VPN failure.

**Steps:**
1. Note baseline routing table: `ip route` (Linux) or `netstat -rn` (macOS)
2. Start CrypRQ VPN
3. Note routing table with VPN active
4. Terminate CrypRQ process
5. Verify routing table returns to baseline (or expected state)

**Classification:**
- **PASS:** Routing table is restored correctly after VPN termination
- **FAIL (Product Bug):** Routing table is left in inconsistent state, causing network issues
- **ENV LIMITATION:** Cannot test due to missing TUN support or insufficient privileges

## Result Classification & Reporting

### Classification Guidelines

Use the same classification system as `docs/CI_E2E_INVESTIGATION_RUNBOOK.md`:

**PASS:**
- Test succeeded as expected
- Behavior matches documented expectations

**FAIL (Product Bug):**
- Test revealed an actual issue in CrypRQ's behavior
- Unexpected crashes, hangs, or incorrect routing
- Performance issues that suggest implementation problems

**ENV LIMITATION:**
- Test could not be run due to environment constraints
- Missing TUN support, no root/admin privileges
- Missing tools (`curl`, `dig`, `iperf3`, etc.)
- TLS/CA certificate issues
- Network connectivity problems unrelated to CrypRQ

### Reporting Results

When reporting test results, include:

1. **Test Environment:**
   - OS version and architecture
   - `cryprq` version/build
   - TUN support status
   - Root/admin privilege status

2. **Test Results:**
   - Classification for each test (PASS/FAIL/ENV LIMITATION)
   - Relevant log snippets
   - Screenshots or output from IP/DNS/WebRTC tests

3. **Artifacts:**
   - Output from `scripts/vpn_host_probe.sh` (if available)
   - CrypRQ process logs
   - Routing table snapshots (before/after)
   - Support bundle from `scripts/support_bundle.sh`

4. **Bug Report:**
   - Use Web Tester Bug Report Draft generator (if applicable)
   - Or follow template in `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md`

### Quick Probe Script

For automated validation, use:

```bash
./scripts/vpn_host_probe.sh
```

This script:
- Checks environment prerequisites (OS, root, TUN)
- Runs basic tunnel start/stop tests
- Optionally runs IP/DNS checks (if enabled)
- Prints a structured summary with classifications

See `scripts/vpn_host_probe.sh` for usage details.

## Leak & Routing Follow-up

After successful host-mode VPN tests, run leak and routing diagnostics:

1. **Run leak probe script**:
   ```bash
   ./scripts/vpn_leak_probe.sh
   ```

2. **Interpret results**:
   - Compare output with `docs/VPN_LEAK_AND_ROUTING_TEST_PLAN.md`
   - Classify results as PASS/FAIL/ENV_LIMITATION based on documented architecture
   - Note any IP, DNS, IPv6, or routing anomalies

3. **Document findings**:
   - Include leak probe output in support bundles when reporting issues
   - Use Web Tester Validation Packet Generator to combine leak test results with session diagnostics
   - Reference leak test results in bug reports when VPN behavior is questioned

The leak probe is informational and helps verify that VPN routing behavior matches documented design. It does not provide anonymity guarantees or full privacy audits.

## Additional Resources

- `docs/VPN_TEST_PLAN.md`: Docker-based P2P tunnel and VPN error handling tests
- `docs/VPN_LEAK_AND_ROUTING_TEST_PLAN.md`: VPN leak & routing test plan
- `docs/ARCHITECTURE_AND_THREAT_MODEL.md`: Architecture overview and threat model
- `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md`: Troubleshooting guide and support bundle collection
- `scripts/vpn_host_probe.sh`: Automated probe script for host-mode validation
- `scripts/vpn_leak_probe.sh`: VPN leak & routing diagnostics probe
- `docs/CI_E2E_INVESTIGATION_RUNBOOK.md`: How to investigate CI/E2E failures

