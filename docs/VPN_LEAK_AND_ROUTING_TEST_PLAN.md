# VPN Leak & Routing Test Plan

## Overview

This document describes procedures for detecting **obvious** IP, DNS, IPv6, and routing leaks when CrypRQ's VPN mode is used on supported hosts (macOS and Linux). These tests focus on **correctness and obvious leak behavior**, not anonymity guarantees or full privacy audits.

**Related Documentation:**
- `docs/ARCHITECTURE_AND_THREAT_MODEL.md`: Threat model assumptions and non-goals
- `docs/VPN_TEST_PLAN.md`: General VPN testing procedures
- `docs/VPN_HOST_TEST_PLAN.md`: Host-mode VPN testing guide
- `scripts/vpn_leak_probe.sh`: Automated probe script for leak diagnostics

## Scope & Non-Goals

### Purpose

This test plan helps validate that CrypRQ's VPN mode behaves as designed on supported hosts by detecting:

- **Obvious IP leaks**: Public IP address unchanged when VPN should be active
- **DNS leaks**: DNS queries bypassing the VPN tunnel when they should not
- **IPv6 leaks**: IPv6 traffic exposing underlying network when it should be tunneled or disabled
- **Routing anomalies**: Traffic not being routed through the VPN interface as expected

### Non-Goals

This test plan does **not** provide:

- **Anonymity guarantees**: CrypRQ's threat model explicitly lists anonymity as a non-goal (see `docs/ARCHITECTURE_AND_THREAT_MODEL.md`)
- **Full privacy audit**: These tests focus on correctness, not comprehensive privacy analysis
- **Kill-switch guarantee**: This plan does not test fail-closed behavior or automatic disconnection on VPN failure
- **WebRTC leak automation**: Browser-based WebRTC leak tests are documented as manual procedures

### Threat Model Assumptions

For detailed assumptions and non-goals, see `docs/ARCHITECTURE_AND_THREAT_MODEL.md`. Key points:

- CrypRQ is designed for **confidentiality** (encrypted tunnel) and **integrity** (tamper detection), not anonymity
- VPN mode provides network-layer routing through the encrypted tunnel
- Leak tests verify that routing behavior matches documented design, not that leaks are impossible under all conditions

## Prerequisites & Tools

### Supported Platforms

- **macOS**: Requires root/admin privileges and TUN support
- **Linux**: Requires root privileges and TUN support

### Required Tools

- **curl**: For IP address checks (IPv4 and IPv6 endpoints)
- **dig** or **nslookup**: For DNS resolution checks
- **ip** (Linux) or **ifconfig/route** (macOS): For local routing and interface inspection

### Environment Limitations

The following conditions should be classified as **ENV LIMITATION**, not product bugs:

- TLS/SSL certificate errors when accessing IP-echo services
- Captive portal interference
- No external network access
- Missing tools (dig, curl, etc.)
- Insufficient privileges to inspect routes or interfaces
- Containerized environments where routing inspection is limited

## IP Address Leak Checks

### Procedure

1. **Record baseline IP** (before VPN):
   ```bash
   curl -s https://api.ipify.org
   curl -s https://api64.ipify.org
   ```

2. **Establish CrypRQ VPN** in host-mode:
   ```bash
   sudo ./target/release/cryprq --vpn --listen /ip4/0.0.0.0/udp/9999/quic-v1
   ```

3. **Record VPN IP** (after VPN):
   ```bash
   curl -s https://api.ipify.org
   curl -s https://api64.ipify.org
   ```

4. **Compare results**:
   - If VPN exit host is remote: IP should change to the exit host's IP
   - If VPN is local (loopback): IP may remain the same (expected for local deployments)
   - If VPN is peer-to-peer: IP behavior depends on peer configuration

### Classification

- **PASS**: IP clearly changes to VPN exit IP (when exit host is remote) OR behaves as described in architecture documentation for this deployment
- **FAIL**: IP remains the same when it should change, or obviously reveals underlying ISP IP against documented expectations
- **ENV LIMITATION**: TLS/CA errors, captive portal interference, no external network access, or unknown exit behavior

### Notes

- This test verifies **expected behavior**, not anonymity by itself
- Local/loopback VPN deployments may not change public IP
- Results must be interpreted in context of deployment architecture

## DNS Behavior & Leak Checks

### Concept

A **DNS leak** occurs when DNS queries go to resolvers outside the VPN path when the VPN should own DNS resolution. This can expose which domains a user is querying to their ISP or other third parties.

### Procedure

1. **Capture baseline DNS resolvers** (before VPN):
   - **Linux**: `cat /etc/resolv.conf`
   - **macOS**: `scutil --dns` or `networksetup -getdnsservers Wi-Fi`
   - **systemd-resolved** (Linux): `resolvectl status`

2. **Establish CrypRQ VPN** in host-mode

3. **Capture DNS resolvers** (after VPN):
   - Repeat step 1 commands
   - Note any changes in resolver configuration

4. **Test DNS resolution** (optional):
   ```bash
   dig example.com +trace
   dig whoami.cloudflare @1.1.1.1
   ```
   - Observe which resolver answers the query
   - Check if queries go through VPN interface or bypass it

### Classification

- **PASS**: DNS behavior matches documented design (e.g., all DNS via VPN exit, or documented split DNS behavior)
- **FAIL**: DNS clearly bypasses VPN when documentation says it should not
- **ENV LIMITATION**: dig/nslookup not installed, systemd-resolved complexity, unclear resolver behavior, or TLS/CA errors

### Notes

- DNS behavior depends on VPN configuration and OS DNS management
- Some deployments may intentionally use split DNS (e.g., local domains via local resolver, external via VPN)
- Results must be interpreted against documented architecture

## IPv6 Behavior & Leak Checks

### Concept

IPv6 leaks can expose a user's real network even if IPv4 appears to be "VPN'd." IPv6 addresses are often tied to the underlying network infrastructure and can reveal location or ISP information.

### Procedure

1. **Check baseline IPv6** (before VPN):
   ```bash
   # Linux
   ip -6 addr show
   curl -6 https://api64.ipify.org
   
   # macOS
   ifconfig | grep inet6
   curl -6 https://api64.ipify.org
   ```

2. **Establish CrypRQ VPN** in host-mode

3. **Check IPv6 after VPN**:
   - Repeat step 1 commands
   - Note whether IPv6 addresses changed or were disabled

### Classification

- **PASS**: Behavior matches documented design (e.g., IPv6 disabled at OS level, or fully tunneled through VPN)
- **FAIL**: IPv6 remains exposed to underlying ISP when design says it should be hidden/tunneled
- **ENV LIMITATION**: No IPv6 connectivity, TLS/CA failures, or unclear IPv6 behavior

### Notes

- Some VPN deployments disable IPv6 entirely to prevent leaks
- Others tunnel IPv6 through the VPN interface
- Results must match documented architecture expectations

## Routing & Split Behavior

### Concept

**Split tunneling** allows some traffic to go through the VPN while other traffic uses the direct connection. **Full tunneling** routes all traffic through the VPN. Routing behavior affects which traffic is protected and which is exposed.

### Procedure

1. **Inspect baseline routes** (before VPN):
   ```bash
   # Linux
   ip route show
   ip route show default
   
   # macOS
   netstat -rn | grep default
   route -n get default
   ```

2. **Establish CrypRQ VPN** in host-mode

3. **Inspect routes after VPN**:
   - Repeat step 1 commands
   - Note changes in default routes and interface priorities
   - Check if VPN interface (e.g., `cryprq0`) appears in routing table

4. **Test split tunneling** (if applicable):
   - For deployments with split tunneling, test a "tunneled" destination vs "non-tunneled" destination:
     ```bash
     curl -v https://tunneled-destination.example.com
     curl -v https://non-tunneled-destination.example.com
     ```
   - Verify that traffic routing matches documented split-tunneling configuration

### Classification

- **PASS**: Routing changes and split behavior match architecture documentation
- **FAIL**: Routes don't change where they should, or split tunneling behaves contrary to configuration
- **ENV LIMITATION**: Insufficient privileges to inspect routes, containerized environments, missing tools, or unclear routing behavior

### Notes

- Routing behavior depends on VPN configuration and OS routing policies
- Some deployments may use policy-based routing (e.g., specific destinations via VPN, others direct)
- Results must be interpreted against documented architecture

## WebRTC / Browser Leak Notes (Manual)

WebRTC can leak local IP addresses through ICE (Interactive Connectivity Establishment) candidates, even when a VPN is active. Browser-based tests are required to detect these leaks.

### Manual Procedure

1. **Establish CrypRQ VPN** in host-mode

2. **Open a WebRTC leak test page** in your browser:
   - Examples: "What's My IP Live" WebRTC leak test, "BrowserLeaks" WebRTC test
   - These pages use JavaScript to enumerate ICE candidates and display local/real IPs

3. **Inspect ICE candidates**:
   - Look for local IP addresses (e.g., 192.168.x.x, 10.x.x.x, 172.16-31.x.x)
   - Check if real public IP addresses are exposed
   - Note whether candidates show VPN interface IPs or underlying network IPs

4. **Classification**:
   - **PASS**: Only VPN interface IPs or expected local IPs are exposed
   - **WARN**: Some local IPs exposed, but not obviously leaking real public IP
   - **FAIL**: Real public IP addresses clearly exposed when VPN should hide them
   - **ENV LIMITATION**: Browser restrictions, JavaScript disabled, or unclear results

### Notes

- WebRTC leak tests are **manual** and out-of-scope for automation in this phase
- Browser behavior varies by vendor and version
- Some browsers may require configuration changes to prevent WebRTC leaks
- Results should be documented alongside IP/DNS/IPv6 checks

## Result Classification & Reporting

### Classification Guidelines

Follow existing patterns from other test plans:

- **PASS**: Behavior matches documented architecture and expectations
- **FAIL**: Clear violation of documented behavior (e.g., IP unchanged when it should change, DNS bypassing VPN when it should not)
- **ENV LIMITATION**: Environment constraints prevent proper testing (missing tools, TLS/CA errors, insufficient privileges, containerized environments)
- **WARN**: Ambiguous results that require human interpretation or may indicate issues but are not clearly failures

### Reporting Tools

Use existing CrypRQ reporting tools to document leak test results:

- **vpn_leak_probe.sh**: Quick automated diagnostics for IP/DNS/IPv6/routing observations
- **Web Tester Validation Packet**: Generate comprehensive session reports including leak test notes
- **Support Bundle** (`scripts/support_bundle.sh`): Collect system diagnostics alongside leak probe output
- **Bug Report Draft**: Include leak test results when filing bugs about VPN behavior

### Example Report Snippet

```markdown
## VPN Leak & Routing Test Results

**Test Date:** 2025-01-XX
**Platform:** macOS 14.x
**CrypRQ Version:** v1.0.x

### IP Address Checks
- **Before VPN:** 203.0.113.42
- **After VPN:** 198.51.100.15
- **Result:** PASS (IP changed to VPN exit host)

### DNS Checks
- **Before VPN:** 8.8.8.8, 8.8.4.4
- **After VPN:** 198.51.100.1 (VPN exit resolver)
- **Result:** PASS (DNS routed through VPN)

### IPv6 Checks
- **Before VPN:** 2001:db8::1 (ISP-assigned)
- **After VPN:** IPv6 disabled
- **Result:** PASS (IPv6 disabled as documented)

### Routing Checks
- **Default route:** Changed to cryprq0 interface
- **Split tunneling:** Not configured (full tunnel)
- **Result:** PASS (routing matches architecture)

### WebRTC Checks (Manual)
- **ICE candidates:** Only VPN interface IPs visible
- **Result:** PASS (no obvious leaks)

**Overall Classification:** PASS
**Notes:** All checks passed. VPN behavior matches documented architecture.
```

## Integration with Other Test Plans

This leak & routing test plan complements:

- **VPN_TEST_PLAN.md**: General VPN testing procedures (Docker and host-mode)
- **VPN_HOST_TEST_PLAN.md**: Detailed host-mode VPN testing guide
- **CI_E2E_INVESTIGATION_RUNBOOK.md**: CI/E2E failure investigation procedures
- **TROUBLESHOOTING_SUPPORT_BUNDLE.md**: Support bundle collection and bug reporting

When running comprehensive VPN tests, follow this sequence:

1. Run `scripts/vpn_host_probe.sh` for basic VPN functionality validation
2. Run `scripts/vpn_leak_probe.sh` for leak diagnostics
3. Perform manual WebRTC leak tests in browser
4. Document results using Validation Packet Generator or support bundle
5. Compare results against architecture documentation and classify as PASS/FAIL/ENV LIMITATION

## References

- [CrypRQ Architecture & Threat Model](ARCHITECTURE_AND_THREAT_MODEL.md)
- [VPN Test Plan](VPN_TEST_PLAN.md)
- [VPN Host Test Plan](VPN_HOST_TEST_PLAN.md)
- [Troubleshooting & Support Bundle Guide](TROUBLESHOOTING_SUPPORT_BUNDLE.md)

