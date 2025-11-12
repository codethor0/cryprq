# Cross-Platform QA Playbook

A reproducible checklist for validating CrypRQ across Android, iOS, macOS, Windows, and Linux builds. Focuses on control-plane resilience, DNS handling, mobility scenarios, and crash resistance.

## Test Matrix

| Scenario | Android | iOS | macOS | Windows | Linux |
|----------|---------|-----|-------|---------|-------|
| DNS leak test |  |  |  |  |  |
| IPv6 connectivity |  |  |  |  |  |
| Captive portal handling |  |  |  |  |  |
| Airplane mode / network flip |  |  |  |  |  |
| Roaming (cell ↔ Wi-Fi) |  |  | N/A | N/A | N/A |
| App/host restart |  |  |  |  |  |
| Battery optimization |  | N/A | N/A | N/A | N/A |
| Crash resilience |  |  |  |  |  |
| Metrics/health endpoint |  |  |  |  |  |

## Common Setup

- Deploy two CrypRQ peers (listener + dialer) on a controlled network.
- Configure allowlists and rotation interval (`CRYPRQ_ROTATE_SECS=60`) to make logs easier to inspect.
- Enable metrics (`--metrics-addr 127.0.0.1:9464`).
- Collect logs via platform-specific tools (adb logcat, Console.app, Event Viewer, journald).

## Test Cases

### 1. DNS Leak

**Goal:** Ensure DNS queries traverse the tunnel (or remain local when expected).

Steps:
1. Start listener with metrics enabled.
2. Start tunnel on device.
3. Run `dnsleaktest.com` or `drill example.com` with Wireshark capturing on physical interface.
4. Confirm DNS packets are intercepted/relayed appropriately.

Expected:
- No DNS queries on external interface when tunnel is active.
- Metrics show handshake success and active peer count.

### 2. IPv6 Handling

Steps:
1. Configure tunnel to include IPv6 route (`addRoute("2001:/64")`).
2. Use `ping6` or `curl -6 https://ipv6.google.com`.

Expected:
- IPv6 packets traverse the tunnel.
- No address conflict; system retains global IPv6 connectivity.

### 3. Captive Portal

Steps:
1. Use a captive portal simulator or hotspot requiring login.
2. Connect device, start tunnel.
3. Observe behavior before and after portal authentication.

Expected:
- Tunnel handles authentication interruptions (retries after network change).
- Logs show `on_network_change` invocation (Android) or `handleAppMessage` (iOS).

### 4. Airplane Mode / Network Flip

Steps:
1. Start tunnel.
2. Toggle airplane mode or disable Wi-Fi for 15 seconds.
3. Re-enable connectivity.

Expected:
- Tunnel disconnects gracefully, then reconnects automatically.
- No crash or leaked file descriptors.
- Metrics show reconnection (handshake count increments).

### 5. Roaming (Android/iOS)

Steps:
1. Start tunnel on cellular.
2. Switch to Wi-Fi (and vice versa) while pinging peer.

Expected:
- Packet pump adjusts to new interface.
- CrypRQ reconnects without manual intervention.

### 6. App/Host Restart

Steps:
1. Force-stop app / kill process.
2. Relaunch and reconnect.

Expected:
- Stale handles cleaned up.
- Allowlist persists via local storage.
- No errors in logs.

### 7. Battery Optimization (Android)

Steps:
1. With tunnel active, enable system battery saver.
2. Observe after 15 minutes.

Expected:
- Foreground service keeps VPN alive.
- Notification remains visible.

### 8. Crash Resilience

Steps:
1. Intentionally trigger error (invalid peer ID) to ensure graceful failure.
2. Inspect logs for friendly error message.

Expected:
- No unhandled exceptions.
- UI reports failure with actionable guidance.

### 9. Metrics/Health Endpoint

Steps:
1. Query `http://127.0.0.1:9464/metrics` while tunnel runs.
2. Verify rotation counters, handshake stats, active peers.
3. Check `/healthz` returns `ok`.

Expected:
- Metrics reflect recent events.
- Health endpoint returns `503` until initialization completes.

## Logging Expectations

- Rotation logs: `event=key_rotation`.
- Denied peers: `event=peer_denied`.
- Backoff warnings: `event=inbound_backoff`.
- Ensure logs include timestamps in UTC.

## QA Artifacts

- Save logs to `release-*/qa/platform/`.
- Capture metrics snapshots.
- Update `finish_qa_and_package.sh` to bundle multi-platform logs.

## Regression Criteria

- Any crash, unhandled exception, or stuck state fails QA.
- DNS leaks or IPv6 failures block release.
- Metrics/health endpoints must respond; missing data is a regression.

## Tooling

- Android: `adb`, `adb shell dumpsys battery`, `adb shell ping`.
- iOS/macOS: `networkQuality`, `packet tunnel provider logging`, `Console.app`.
- Windows: `pktmon`, `netsh trace`.
- Linux: `tcpdump`, `nmcli`, `systemd-journald`.

## Sign-off

- QA lead signs off once all scenarios pass on supported platforms.
- Document results in `docs/qa_reports/<version>.md`.

