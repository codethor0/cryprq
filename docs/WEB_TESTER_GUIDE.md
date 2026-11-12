# CrypRQ Web Tester Guide

## Overview

The CrypRQ Web Tester is a web-based interface for testing CrypRQ's P2P encrypted tunnel functionality. It provides a browser-based harness that allows you to establish encrypted connections, monitor connection health, and export session logs without using the CLI directly.

In the Docker/web environment, VPN mode is primarily for error-handling demonstration. The P2P encrypted tunnel mode is fully functional and provides the same ML-KEM (Kyber768) + X25519 hybrid encryption as the CLI.

## Starting the Web Tester

### Prerequisites

- Docker and Docker Compose installed
- Port 8787 available on localhost

### Starting the Stack

From the repository root:

```bash
docker compose -f docker-compose.web.yml up --build -d
```

This command will:
- Build the web UI container
- Start the Node.js backend server
- Serve the React frontend
- Make the UI available at http://localhost:8787

### Accessing the UI

Open your web browser and navigate to:

```
http://localhost:8787
```

You should see:
- CrypRQ Web Tester header
- Connection status indicator (Connected/Disconnected)
- Guided Tests panel
- Connection controls (Mode, Port, Peer Address, VPN Mode)
- Encryption Status panel
- Connection Health panel
- Test Summary panel
- Debug Console

### Stopping the Stack

To stop the web tester:

```bash
docker compose -f docker-compose.web.yml down --remove-orphans
```

## Guided Tests

The Guided Tests panel provides three pre-configured scenarios to help you test common use cases.

### Local Loopback P2P Test (Single Machine)

**Purpose:** Test P2P tunnel functionality on a single machine using two browser tabs.

**Steps:**

1. Click "Apply Settings" in the "Local Loopback P2P Test" card
2. This sets:
   - Mode: Listener
   - Port: 10000
   - Peer Address: /ip4/127.0.0.1/udp/10000/quic-v1 (when in Dialer mode)

3. **Tab 1 (Listener):**
   - Ensure mode is set to "Listener"
   - Port should be 10000
   - Click "Connect"
   - Wait for "listener_ready" events in the Debug Console

4. **Tab 2 (Dialer):**
   - Switch mode to "Dialer"
   - Peer address should be /ip4/127.0.0.1/udp/10000/quic-v1
   - Click "Connect"
   - Watch for "handshake_complete" and "connection_established" events

**Expected Results:**

- Connection Health panel shows "Encrypted tunnel active"
- Last Handshake shows direction (Inbound for Listener, Outbound for Dialer)
- Debug Console shows encryption events and key rotation
- Test Summary shows "Connected (encrypted tunnel active)"

### Two-Tab Web Test

**Purpose:** Detailed step-by-step connection test with explicit configuration.

**Steps:**

1. Click "Pre-fill Settings" in the "Two-Tab Web Test" card
2. This pre-fills:
   - Port: 10000
   - Peer Address: /ip4/127.0.0.1/udp/10000/quic-v1

3. **Tab 1 - Listener:**
   - Set mode to "Listener"
   - Port: 10000
   - Click "Connect"
   - Verify Debug Console shows:
     - `[STATUS] Connect initiated: listener on port 10000`
     - `[PEER] event=listener_ready ... status=accepting_connections`
     - `[PEER] Local peer id: <peer-id>`

4. **Tab 2 - Dialer:**
   - Set mode to "Dialer"
   - Port: 10000
   - Peer Address: /ip4/127.0.0.1/udp/10000/quic-v1
   - Click "Connect"
   - Verify Debug Console shows:
     - `[STATUS] Connect initiated: dialer on port 10000`
     - `[PEER] event=handshake_complete ... encryption=libp2p-transport cryprq_handshake=none`
     - `[PEER] event=connection_established ... encryption_active=true`

**Expected Results:**

- Both tabs show "Encrypted tunnel active" in Connection Health
- Test Summary shows handshake direction and ping latency
- Key rotation events appear in Debug Console (filter: Rotation)
- No errors in Debug Console (filter: Errors)

### VPN Demo (Error Handling)

**Purpose:** Demonstrate VPN mode error handling in Docker/web environment.

**Steps:**

1. Click "Enable VPN Mode" in the "VPN Demo (Error Handling)" card
2. This enables VPN mode toggle
3. Set mode to "Listener" or "Dialer" (your choice)
4. Click "Connect"

**Expected Behavior:**

- VPN mode toggle is enabled
- Connection attempt is made with `--vpn` flag
- Process fails with clear error message:
  - "VPN mode requires administrator privileges"
  - "Failed to create TUN device (requires root/admin privileges)"
- Error appears in Debug Console with `[ERROR]` prefix
- VPN error panel appears below VPN mode toggle
- P2P encrypted tunnel functionality remains available (you can disable VPN mode and reconnect)

**Key Points:**

- This demonstrates graceful error handling
- The UI clearly indicates that VPN mode failed
- P2P mode (without VPN) continues to work normally
- In a real environment with root/TUN support, VPN mode would succeed

## Observability and Logs

### Connection Health Panel

The Connection Health panel displays real-time metrics derived from log events:

- **Connection State:** Idle, Connecting, or Encrypted tunnel active
- **Last Handshake:** Direction (Inbound/Outbound) and timestamp
- **Last Ping Latency:** Most recent ping measurement (e.g., "86.167µs")
- **Current Epoch:** Key rotation epoch number
- **Rotation Interval:** Time between key rotations (e.g., "Every 300s")

These metrics update automatically as events occur in the Debug Console.

### Test Summary Panel

The Test Summary provides a compact overview of the current test session:

- **Last Run Status:** Overall connection state (Connected, Failed, Idle, etc.)
- **Last Handshake:** Same as Connection Health
- **Last Ping:** Same as Connection Health
- **Key Rotation:** Current epoch and interval
- **Errors This Session:** Count of error-level log entries

Use this panel to quickly assess test results without scrolling through logs.

### Debug Console

The Debug Console shows all log events from the CrypRQ backend.

**Filters:**

- **All:** Show all events
- **Errors:** Show only error-level events
- **Rotation:** Show only key rotation events
- **Peer:** Show only peer/connection events
- **Status:** Show only status/info events

**Download Logs:**

Click "Download Logs" to export all current log entries to a text file:
- Filename format: `cryprq-session-logs-YYYYMMDD-HHMMSS.txt`
- Includes header with timestamp, mode, port, VPN mode status
- All log entries in chronological order
- Useful for bug reports or detailed analysis

**Log Format:**

Each log entry shows:
- `[LEVEL]` prefix (ERROR, STATUS, ROTATION, PEER, INFO)
- Event text (structured log messages from backend)

### Encryption Status Panel

Shows:
- Connection status (Active, Waiting, Connecting, etc.)
- Encryption method (ML-KEM (Kyber768) + X25519 hybrid)
- Local Peer ID (when available)
- Connected Peer ID (when connected)
- Key Epoch and Rotation Interval
- Proof of encryption indicators

## Troubleshooting

### Port and Peer Address Validation

The UI validates inputs before allowing connection:

- **Port:** Must be between 1 and 65535
- **Peer Address (Dialer mode):** Required, must contain `/udp/` and end with `/quic-v1`

**Common Errors:**

- "Port must be between 1 and 65535": Enter a valid port number
- "Peer address is required for Dialer mode": Enter a peer address when in Dialer mode
- "Peer address should be a QUIC multiaddr": Use format like `/ip4/127.0.0.1/udp/10000/quic-v1`

### Connection Failures

**Dialer started before Listener:**

- Error: "Dial error: Transport(... HandshakeTimedOut ...)"
- Solution: Start Listener first, wait for "listener_ready", then start Dialer

**Wrong IP or Port:**

- Error: Connection timeout or refused
- Solution: Verify IP address and port match between Listener and Dialer
- For local testing, use 127.0.0.1 (not 0.0.0.0)

**Port Already in Use:**

- Error: "Address already in use"
- Solution: Choose a different port, or stop the process using the port

### UI Issues

**Console not updating:**

- Check server connection indicator (top right)
- Refresh the page if connection is "Disconnected"
- Check browser console for JavaScript errors

**Filters not working:**

- Ensure you're using the filter buttons in Debug Console toolbar
- Check that events match the selected filter level

**Download Logs button disabled:**

- No events have been logged yet
- Start a connection to generate logs

### Environment Limitations

Some errors are environment-specific and not product bugs:

- **TLS/SSL certificate errors:** Common in CI/sandbox environments with restricted network access
- **npm permission errors (EPERM):** Sandbox restrictions, not application bugs
- **VPN mode failures:** Expected in Docker/web environment without root/TUN support

## Session History & Comparison

The Web Tester includes a Session History feature that allows you to save, view, and compare multiple sessions.

### Saving Sessions

1. After establishing a connection and collecting metrics, click **"Save Current Session"** in the Session History panel.
2. The session (including snapshot and timeline) will be saved to browser localStorage with an auto-generated label like "Session YYYY-MM-DD HH:MM:SS".
3. You can rename sessions by clicking on the label in the history table.

### Viewing Saved Sessions

- The Session History panel displays all saved sessions in a table format.
- Each session shows:
  - Label (click to rename)
  - Creation timestamp
  - Actions (Select Left/Right for comparison, Delete)

### Comparing Sessions

1. Use the Session Comparison panel to compare two saved sessions.
2. Select a **Baseline Session (Left)** and a **Candidate Session (Right)** from the dropdowns.
3. The comparison view shows:
   - **Configuration differences**: Mode, port, peer address, VPN toggle
   - **Metrics differences**: Handshake count, ping averages, key rotation epochs, file transfer status
   - **Diagnostics differences**: Severity counts (ENV LIMITATION, PRODUCT SUSPECT, INFO, UNKNOWN) and hint IDs
4. Click **"Copy Comparison (Markdown)"** to export a formatted comparison report.

### Notes

- Session history is stored locally in your browser (localStorage). It is not sent to any server.
- Maximum of 50 sessions are stored; older sessions are automatically removed when the limit is reached.
- Use Session History & Comparison alongside:
  - Bug Report Draft panel (for detailed bug reports)
  - Timeline export JSON (for event-level analysis)
  - Support bundle script (`scripts/support_bundle.sh`) and validation run reports (`scripts/validation_run_report.sh`) for comprehensive diagnostics

## Best Practices

1. **Start with Guided Tests:** Use the Guided Tests panel to ensure correct configuration
2. **Monitor Connection Health:** Watch the Connection Health panel for real-time status
3. **Use Debug Console Filters:** Filter logs by type to focus on relevant events
4. **Export Logs for Analysis:** Download logs when reporting issues or analyzing behavior
5. **Test in Two Tabs:** Use separate browser tabs for Listener and Dialer to simulate real-world usage
6. **Verify Encryption:** Check Encryption Status panel to confirm ML-KEM+X25519 is active
7. **Watch Key Rotation:** Monitor Rotation filter to see key rotation events (every 300s by default)
8. **Save Sessions for Comparison:** Use Session History to track regressions and improvements over time

## Scenario Playbooks

The Scenario Playbook feature allows you to generate a reproducible Markdown document from any session (current or saved) for regression testing and bug triage.

### Generating a Playbook

1. Use the **Scenario Playbook** panel in the Web Tester.
2. Choose your source:
   - **Use current session**: Generates a playbook from the active session
   - **Use stored session**: Select a saved session from Session History
3. Optionally provide a custom label for the playbook.
4. The playbook Markdown will be displayed in the preview area.
5. Click **"Copy Playbook (Markdown)"** to copy the full playbook to your clipboard.

### Playbook Contents

Each playbook includes:

- **Environment & Configuration**: Mode, port, peer address, VPN toggle, CLI command
- **Steps to Reproduce**: Numbered steps to recreate the scenario
- **Expected Signals**: Connection state, metrics, log patterns, file transfer outcomes
- **Diagnostics & Classification**: Diagnostic hints with severity classification (ENV LIMITATION vs PRODUCT SUSPECT)
- **Notes & Variants**: Placeholder for manual notes and source session information

### Use Cases

- **Regression Testing**: Save a playbook before making changes, then compare results after changes
- **Bug Triage**: Share exact repro steps and expected signals with maintainers
- **Documentation**: Create reproducible test scenarios for CI/CD or manual testing

### Integration with Other Tools

Scenario Playbooks complement:

- **Session History & Comparison**: Compare playbooks from different sessions
- **Bug Report Draft**: Include playbook Markdown in bug reports
- **Timeline Export JSON**: Use timeline data for detailed event analysis
- **Support Bundle** (`scripts/support_bundle.sh`): Combine playbook with system diagnostics
- **Validation Run Reports** (`scripts/validation_run_report.sh`): Include playbooks in validation documentation

## Validation Packets (All-In-One Session Report)

The Validation Packet feature generates a comprehensive Markdown document that combines all session information into a single, shareable report suitable for issues, internal tickets, and release notes.

### Generating a Validation Packet

1. Use the **Validation Packet** panel in the Web Tester.
2. Choose your source:
   - **Use current session**: Generates a packet from the active session
   - **Use stored session**: Select a saved session from Session History
3. Optionally select a baseline:
   - Choose a baseline session to include Regression Guard comparison
   - The packet will include regression status (PASS/WARN/FAIL) and signal details
4. Configure attachments:
   - **Attach Scenario Playbook**: Include the scenario playbook in the packet
   - **Attach Bug Report Draft**: Include bug report draft if available
5. Optionally provide:
   - **Packet Label**: A human-friendly name (e.g., "1.0.3-pre-release-smoke-run")
   - **Notes**: Additional context or observations
6. The packet Markdown will be displayed in the preview area.
7. Click **"Copy Validation Packet (Markdown)"** to copy the full packet to your clipboard.

### Packet Contents

Each Validation Packet includes:

- **Summary**: Session type, timestamps, baseline info, regression status
- **Environment & Configuration**: Mode, port, peer address, VPN toggle, CLI command, environment info
- **Connection & Metrics Overview**: Connection state, latency stats, key rotation info, error counts, file transfer status
- **Diagnostics Summary**: Diagnostic counts by severity and key diagnostic hints
- **Regression Guard Summary**: Regression status, signals, and metric comparison (if baseline selected)
- **Scenario Playbook**: Reproducible scenario document (if attached)
- **Bug Report Draft**: Bug report template (if attached)
- **Timeline & Artifacts References**: Event summary and guidance on additional artifacts

### Use Cases

- **Issue Reporting**: Paste the Validation Packet into GitHub issues or internal tickets for comprehensive context
- **Release Documentation**: Generate packets for release validation runs to document system state
- **Regression Tracking**: Include baseline comparison to show regressions or improvements
- **Internal Documentation**: Use packets to document test scenarios and system behavior

### Integration with Other Tools

Validation Packets complement and can be used alongside:

- **Timeline JSON Export**: Detailed event-level analysis
- **Raw Logs Export**: Full log history
- **Support Bundle Tarball** (`scripts/support_bundle.sh`): Comprehensive system diagnostics
- **CLI Validation Run Report** (`scripts/validation_run_report.sh`): Automated probe outputs
- **Regression Guard Reports**: Can be integrated into packets or used separately
- **Scenario Playbooks**: Can be attached to packets or used independently

### Notes

- Playbooks are generated entirely in the browser (no network calls)
- They use only data from the selected session snapshot and timeline
- Missing data fields are marked as "N/A" or "<not recorded>"
- Playbooks can be saved as Markdown files or pasted into issue trackers

## Regression Guard & Baselines

The Regression Guard feature allows you to compare the current session against a "golden baseline" to detect regressions and classify them as PASS/WARN/FAIL.

### Marking a Baseline

1. Save a session in Session History (see Session History & Comparison section above).
2. In the Session History panel, click **"Mark Baseline"** for the session you want to use as a baseline.
3. Optionally provide a baseline tag (e.g., "1.0.2-golden-e2e") when prompted.
4. The session will be marked with a **BASELINE** badge in the Session History table.

### Using Regression Guard

1. Use the **Regression Guard** panel in the Web Tester.
2. Select a baseline from the dropdown (only sessions marked as baselines will appear).
3. The panel will automatically compare the current session against the selected baseline.
4. Review the status:
   - **PASS**: No regressions detected; session matches baseline expectations
   - **WARN**: Performance or environmental differences detected (may be noise)
   - **FAIL**: Critical regressions detected (e.g., new PRODUCT SUSPECT diagnostics, connection failures)
5. Review the regression signals to understand what changed.
6. Click **"Copy Report (Markdown)"** to export a formatted regression report.

### Interpreting Status

- **PASS**: All metrics and diagnostics match baseline expectations.
- **WARN**: Often reflects:
  - Performance variation (latency, throughput) that may be environmental
  - Increased ENV LIMITATION diagnostics (not product bugs)
  - Network noise or timing differences
- **FAIL**: Indicates:
  - New PRODUCT SUSPECT diagnostics not present in baseline
  - Connection state regressions (baseline connected, current did not)
  - File transfer failures that succeeded in baseline

### Integration with Other Tools

Regression Guard Reports complement:

- **Scenario Playbook**: Use playbook for reproducible steps, regression guard for before/after comparison
- **Bug Report Draft**: Include regression guard report when filing bugs about regressions
- **Session Comparison**: Compare any two sessions; regression guard compares specifically against baselines
- **Timeline JSON**: Use timeline export for detailed event-level analysis
- **Support Bundle** (`scripts/support_bundle.sh`): Combine regression guard report with system diagnostics
- **Validation Run Reports** (`scripts/validation_run_report.sh`): Include regression guard status in validation documentation

## Additional Resources

- `docs/QUICKSTART_CLI.md`: CLI usage guide
- `docs/VPN_TEST_PLAN.md`: Comprehensive testing procedures
- `docs/SYSTEM_VPN_IMPLEMENTATION.md`: VPN implementation details
- `README.md`: Project overview and build instructions

