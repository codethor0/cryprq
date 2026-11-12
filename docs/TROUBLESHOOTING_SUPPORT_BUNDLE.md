# CrypRQ Troubleshooting and Support Bundle Guide

## Overview

This document helps users and developers diagnose issues with CrypRQ and collect actionable evidence for bug reports. It covers the current P2P tunnel and Web Tester release, focusing on practical troubleshooting steps and support bundle collection.

This guide is for the P2P encrypted tunnel functionality. For OS-level VPN troubleshooting (which requires root privileges and TUN device support), see `docs/SYSTEM_VPN_IMPLEMENTATION.md` and `docs/VPN_TEST_PLAN.md`.

## One-Command Support Bundle

The fastest way to collect a complete support bundle is to run the automated collection script:

```bash
./scripts/support_bundle.sh
```

Or if the script is not executable:

```bash
bash scripts/support_bundle.sh
```

### What the Script Does

The support bundle script collects:

- **Git metadata**: Current branch, status, recent commits, HEAD commit hash
- **Health checks**: Rust formatting check, governance shield results (with ENV LIMITATION notes)
- **Docker state**: Running containers, container status, limited logs from cryprq-web and E2E containers
- **Web build artifacts**: Summary of web/dist directory structure
- **Documentation snapshot**: Copies relevant troubleshooting and quickstart docs

The script creates a compressed tarball at:

```
support_bundles/cryprq_support_YYYYMMDD_HHMMSS.tar.gz
```

### What the Script Does NOT Collect

The script is designed to be safe and privacy-preserving:

- Does not collect environment variables or shell history
- Does not collect files from outside the repository
- Does not collect Docker environment variables
- Does not inspect system network interfaces or routes
- Does not stop or modify any running containers

### Using the Support Bundle

After running the script, attach the generated `.tar.gz` file to bug reports along with:

- Screenshots of Connection Health, Advanced Metrics, and Test Summary panels
- Description of the issue and steps to reproduce
- Time of issue and configuration details

See the "What to Attach in a Bug Report" section below for complete details.

## Manual Collection Steps (If You Cannot Run the Script)

If you cannot run the automated support bundle script, you can manually collect the same information using the steps below. The automated script is preferred as it ensures consistency and completeness.

## Quick Health Check

The fastest way to verify your CrypRQ setup is healthy is to run the quick smoke script.

### Running the Smoke Script

From the repository root:

```bash
./scripts/quick_smoke.sh
```

Or if the script is not executable:

```bash
bash scripts/quick_smoke.sh
```

### What the Smoke Script Does

The script runs through several phases:

**Phase 0: Repo Status**
- Checks git working tree status
- Lists branches (should show only main locally and origin/main remotely)

**Phase 1: Static Checks**
- Runs `cargo fmt --all --check` (Rust formatting validation)
- Runs `python3 scripts/governance_shield.py` (formatting, emoji checks, prompt file checks, Node.js build)

**Phase 2: Web Build**
- Builds the web UI with `npm run build` in the `web/` directory
- Verifies TypeScript compilation and Vite bundle generation

**Phase 3: QUIC E2E Docker Harness**
- Brings down any existing E2E containers
- Runs the full E2E test suite via `docker/e2e/docker-compose.e2e.yml`
- Expects: Passed: 5, Failed: 0, Skipped: 0, exit code 0
- Cleans up containers after completion

**Phase 4: Web Stack Bring-Up**
- Brings down any existing web stack containers
- Starts the web stack via `docker-compose.web.yml`
- Verifies container is running and port 8787 is mapped
- Performs HTTP sanity check with curl

### Interpreting Results

**Healthy Run:**
- All phases complete without errors
- E2E tests show "Passed: 5, Failed: 0, Skipped: 0"
- Web container is Up and responding on port 8787
- HTTP check returns 200 OK

**ENV LIMITATION vs Product Failures:**

Some failures are environment-specific and not product bugs:

- **SSL/TLS certificate errors** when accessing `index.crates.io` (crates.io registry)
- **npm EPERM permission errors** in restricted sandbox environments
- **TypeScript type check failures** due to npm permission issues

These are classified as ENV LIMITATION and should be noted but not treated as product bugs.

**Real Product Failures:**
- Rust formatting check fails (indicates code formatting issues)
- E2E tests fail (indicates QUIC/crypto/tunnel problems)
- Web build fails with TypeScript errors (indicates code issues, not permissions)
- Web container fails to start or serve content

## Collecting Logs from the Web Tester

The Web Tester provides built-in log export functionality for troubleshooting.

### Using the Debug Console

1. Open the Web Tester at http://localhost:8787
2. Navigate to the Debug Console (fixed panel at the bottom of the page)
3. Use filters to focus on relevant events:
   - **All**: Show all log events
   - **Errors**: Show only error-level events
   - **Rotation**: Show only key rotation events
   - **Peer**: Show only peer/connection events
   - **Status**: Show only status/info events

### Exporting Logs

1. In the Debug Console, click the "Download Logs" button
2. A text file will be downloaded with filename format: `cryprq-session-logs-YYYYMMDD-HHMMSS.txt`
3. The log file contains:
   - Header with timestamp, mode, port, VPN mode status
   - All log entries in chronological order
   - Event levels (ERROR, STATUS, ROTATION, PEER, INFO)

### What Logs Contain

The exported log file includes:
- Connection establishment events
- Handshake completion events
- Key rotation events
- Ping latency measurements
- File transfer events
- Error messages and stack traces
- VPN mode error messages (if applicable)

### When to Export Logs

Export logs when:
- Reporting a bug or issue
- Investigating connection problems
- Analyzing performance issues
- Verifying encryption behavior
- Troubleshooting file transfer problems

## Capturing Metrics

The Web Tester provides several panels for monitoring metrics and session state.

### Connection Health Panel

Shows real-time connection metrics:
- Connection state (Idle, Connecting, Encrypted tunnel active)
- Last handshake direction and timestamp
- Last ping latency
- Current key rotation epoch
- Rotation interval

**How to Capture:**
- Take a screenshot of the Connection Health panel
- Note the current values when an issue occurs
- Include in bug reports to show connection state at time of issue

### Test Summary Panel

Provides a compact overview of the current test session:
- Last run status
- Last handshake details
- Last ping latency
- Key rotation information
- Errors this session count
- Last file transfer status and metrics

**How to Capture:**
- Take a screenshot of the Test Summary panel
- Note the values when reporting issues
- Use to quickly assess test results

### Advanced Metrics Panel

Shows aggregate session-level statistics:
- Connection attempts (total, succeeded, failed)
- Ping latency statistics (min, max, avg, sample count)
- Key rotation count and last epoch
- File transfer statistics (total, succeeded, failed)
- Error count and last error details

**How to Capture:**
- Take a screenshot of the Advanced Metrics panel
- Include in bug reports to show session-level patterns
- Useful for identifying trends or recurring issues

### File Transfer Metrics

The Test Summary panel includes "Last File Transfer" with:
- Transfer status (Completed, Failed, In progress)
- File size and duration
- Approximate throughput (Mbps)

**How to Capture:**
- Note the Last File Transfer status in Test Summary
- Check file transfer state in the file upload area
- Include transfer metrics when reporting file transfer issues

## Common Troubleshooting Steps

### Connection Issues

**Problem: Dialer cannot connect to Listener**

**Check:**
1. Is the Listener running? (Connection Health should show "Encrypted tunnel active")
2. Is the port correct? (Verify port matches between Listener and Dialer)
3. Is the peer address correct? (For local testing, use `/ip4/127.0.0.1/udp/<PORT>/quic-v1`)
4. Did you start the Listener before the Dialer? (Listener must be ready before Dialer connects)

**Solution:**
- Start Listener first, wait for "listener_ready" in Debug Console
- Then start Dialer with matching port and correct peer address
- Check Debug Console for connection errors

**Problem: "Address already in use"**

**Check:**
- Is another process using the port?
- Did a previous connection not clean up properly?

**Solution:**
- Choose a different port
- Restart the web stack: `docker compose -f docker-compose.web.yml down && docker compose -f docker-compose.web.yml up --build -d`

**Problem: Connection timeout**

**Check:**
- Firewall blocking UDP traffic on the selected port
- Network connectivity between Listener and Dialer
- Correct IP address (use 127.0.0.1 for local testing, not 0.0.0.0)

**Solution:**
- Verify firewall rules allow UDP on the port
- For local testing, ensure both tabs use 127.0.0.1
- Check browser console for network errors

### File Transfer Issues

**Problem: "Not connected to peer"**

**Check:**
- Is Connection Health showing "Encrypted tunnel active"?
- Is the connection established? (Test Summary should show "Connected")

**Solution:**
- Establish connection first (Listener + Dialer both connected)
- Wait for "connection_established" events in Debug Console
- Then attempt file transfer

**Problem: File transfer hangs or times out**

**Check:**
- Is the connection still active? (Connection Health panel)
- Are there errors in Debug Console?
- Is the file size reasonable? (Try a smaller file first)

**Solution:**
- Verify connection is still active
- Check Debug Console for file transfer errors
- Try with a smaller file to rule out size-related issues
- Export logs for analysis

### VPN Mode Issues

**Problem: VPN mode fails with "Failed to create TUN device"**

**Check:**
- Are you running in Docker/web environment? (VPN mode requires root/TUN support)
- Does the error message clearly state "requires root/admin privileges"?

**Solution:**
- This is expected behavior in Docker/web environments without root/TUN support
- VPN mode error handling is working correctly (clear error message, graceful failure)
- P2P encrypted tunnel remains available without VPN mode
- For full VPN functionality, run CLI with sudo on a system with TUN support

**Classification:**
- This is an ENV LIMITATION, not a product bug
- The error message is clear and the process fails gracefully
- P2P tunnel functionality is unaffected

### Browser/UI Issues

**Problem: Console not updating**

**Check:**
- Server connection indicator (top right) - should show "Connected"
- Browser console for JavaScript errors
- Network tab for EventSource connection issues

**Solution:**
- Refresh the page if connection is "Disconnected"
- Check browser console for errors
- Verify web stack is running: `docker ps | grep cryprq-web`

**Problem: Filters not working in Debug Console**

**Check:**
- Are you using the filter buttons in the Debug Console toolbar?
- Do events match the selected filter level?

**Solution:**
- Use the filter buttons (All, Errors, Rotation, Peer, Status)
- Verify events have the correct level prefix

## What to Attach in a Bug Report

When reporting an issue, include the following information:

### Required Information

1. **Time of Issue**
   - When did the problem occur?
   - What were you doing when it happened?

2. **Connection Configuration**
   - Mode (Listener or Dialer)
   - Port number
   - Peer address (if Dialer)
   - VPN mode enabled? (yes/no)

3. **Screenshots**
   - Connection Health panel (showing current state)
   - Advanced Metrics panel (showing session statistics)
   - Test Summary panel (showing test results)
   - Debug Console (filtered to relevant events, especially Errors)

4. **Log Export**
   - Download logs from Debug Console
   - Attach the `cryprq-session-logs-*.txt` file
   - Include logs from the time period when the issue occurred

5. **Quick Smoke Script Output**
   - Run `./scripts/quick_smoke.sh` and capture output
   - Include full output or key error snippets
   - Note any ENV LIMITATION failures separately

### Optional but Helpful

- File transfer test details (if relevant)
- Browser and OS information
- Docker version (if using Docker)
- Network configuration details (if relevant)
- VPN host-mode probe output (if testing VPN mode on real hosts):
  - Run `./scripts/vpn_host_probe.sh` on host machines that hit VPN issues
  - Attach probe output alongside Web Tester bug report draft and support bundle tarball
  - See `docs/VPN_HOST_TEST_PLAN.md` for detailed host-mode testing procedures
- Validation run report (for entire validation runs):
  - Run `./scripts/validation_run_report.sh` to generate a Markdown summary
  - Attach the generated `docs/VALIDATION_RUN_*.md` file alongside the support bundle tarball
  - See `docs/VALIDATION_RUN_TEMPLATE.md` for template structure
- Session History & Comparison snapshot (if using Web Tester):
  - Use the Session Comparison panel to compare two sessions
  - Click "Copy Comparison (Markdown)" to generate a comparison report
  - Attach the comparison Markdown when filing bugs to show regressions or improvements between sessions
- Scenario Playbook (if using Web Tester):
  - Generate a Scenario Playbook from the current or a stored session
  - Click "Copy Playbook (Markdown)" to copy the playbook
  - Paste the playbook Markdown into your bug report alongside:
    - Session comparison Markdown (if comparing before/after)
    - Timeline JSON export
    - Support bundle tarball
  - The playbook provides reproducible steps and expected signals for regression testing and bug triage
- Regression Guard Report (if using Web Tester and comparing against a baseline):
  - Mark a known-good session as a baseline in Session History
  - Use the Regression Guard panel to compare current session against the baseline
  - Click "Copy Report (Markdown)" to generate a regression guard report
  - Include the report when filing bugs about regressions, especially when comparing a failing run to a known-good baseline
  - The report provides PASS/WARN/FAIL classification and detailed regression signals
- Validation Packet (if using Web Tester):
  - Generate a Validation Packet from the current or a stored session
  - Optionally include baseline comparison, scenario playbook, and bug report draft
  - Click "Copy Validation Packet (Markdown)" to copy the comprehensive packet
  - Attach the packet when filing complex issues that require full context
  - Typical recommended set for complex issues:
    - Support bundle tarball
    - Validation run report (CLI)
    - Timeline JSON export
    - Web Tester Validation Packet (Markdown)
- VPN leak probe output (if testing VPN mode on host):
  - Run `./scripts/vpn_leak_probe.sh` before and after starting VPN
  - Include the probe output and any notes from `docs/VPN_LEAK_AND_ROUTING_TEST_PLAN.md`
  - Attach when filing bugs about VPN behavior, IP leaks, DNS leaks, or routing issues
  - The probe collects IP, DNS, IPv6, and routing observations for correctness verification

### Using the Web Tester Bug Report Draft

The Web Tester includes a "Bug Report Draft" panel that automatically generates a bug report template pre-filled with your current session configuration and metrics. This makes it easier to create structured bug reports.

**How to Use:**
1. Open the Web Tester at http://localhost:8787
2. Navigate to the "Bug Report Draft" panel (located near the Advanced Metrics and Test Summary panels)
3. The panel shows a Markdown-formatted bug report draft with:
   - Current configuration (mode, port, peer address, VPN toggle)
   - Connection state and metrics (handshake, ping, rotation, file transfer)
   - Error information (if any)
   - CLI command snippet
4. Click "Copy Draft" to copy the entire report to your clipboard
5. Fill in the placeholders (Summary, Reproduction Steps, Expected Behavior)
6. Attach the requested files:
   - Output from `bash scripts/quick_smoke.sh`
   - Support bundle from `./scripts/support_bundle.sh`
   - Exported logs from the Web Tester ("Download Logs" button)
   - Screenshots of Connection Health, Advanced Metrics, and Test Summary panels

**Note:** The bug report draft generator complements but does not replace `scripts/support_bundle.sh`. The support bundle script collects system-level diagnostics (Docker state, git metadata, health checks) that are not available in the browser. Use both tools together for complete diagnostic information.

### Template for Bug Reports

```
**Issue Description:**
[Brief description of what happened]

**Time of Issue:**
[Timestamp]

**Configuration:**
- Mode: [Listener/Dialer]
- Port: [port number]
- Peer Address: [if Dialer]
- VPN Mode: [enabled/disabled]

**Steps to Reproduce:**
1. [Step 1]
2. [Step 2]
3. [Step 3]

**Expected Behavior:**
[What should have happened]

**Actual Behavior:**
[What actually happened]

**Screenshots:**
[Attach Connection Health, Advanced Metrics, Test Summary, Debug Console]

**Logs:**
[Attach cryprq-session-logs-*.txt file]

**Quick Smoke Output:**
[Attach output from scripts/quick_smoke.sh]

**Environment:**
- OS: [operating system]
- Browser: [browser and version]
- Docker: [version, if applicable]
```

## Additional Resources

- `docs/QUICKSTART_CLI.md`: CLI usage guide
- `docs/WEB_TESTER_GUIDE.md`: Comprehensive Web Tester usage guide
- `docs/FILE_TRANSFER_TEST_PLAN.md`: File transfer testing procedures
- `docs/VPN_TEST_PLAN.md`: VPN and P2P tunnel test procedures
- `docs/SYSTEM_VPN_IMPLEMENTATION.md`: VPN implementation details
- `README.md`: Project overview and build instructions

