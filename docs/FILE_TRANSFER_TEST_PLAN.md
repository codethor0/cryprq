# CrypRQ File Transfer Test Plan

## Overview

This document covers file transfer validation for CrypRQ's P2P encrypted tunnel. File transfer occurs over QUIC transport with ML-KEM (Kyber768) + X25519 hybrid post-quantum encryption, ensuring all file data is encrypted end-to-end between peers.

This test plan focuses on correctness and basic reliability rather than performance benchmarking. All file transfers are validated for integrity and proper encryption behavior.

## Prerequisites

- E2E harness functional (5/5 tests passing via `docker/e2e/docker-compose.e2e.yml`)
- Web Tester running on http://localhost:8787
- Two browser tabs or two separate browsers available for Listener/Dialer testing
- Small test files (e.g., text files, images) and medium-size files (1-5 MB) available for testing

## CLI-Based File Transfer Tests

CLI file transfer is not currently exposed as a standalone feature. File transfer functionality is primarily validated through the Web Tester interface, which uses the same underlying QUIC tunnel and encryption mechanisms.

For CLI-based testing of the core tunnel (which file transfer depends on), see `docs/QUICKSTART_CLI.md`.

## Web Tester File Transfer Tests

### Single Machine, Two-Tab Test

This test validates file transfer between Listener and Dialer running in separate browser tabs on the same machine.

**Setup:**

1. Start the web stack:
   ```bash
   docker compose -f docker-compose.web.yml up --build -d
   ```

2. Open http://localhost:8787 in two browser tabs

**Test Steps:**

**Tab 1 - Listener:**
1. Set mode to "Listener"
2. Set port to 10000
3. Click "Connect"
4. Wait for "listener_ready" events in Debug Console
5. Verify Connection Health shows "Encrypted tunnel active"

**Tab 2 - Dialer:**
1. Set mode to "Dialer"
2. Set port to 10000
3. Set peer address to `/ip4/127.0.0.1/udp/10000/quic-v1`
4. Click "Connect"
5. Wait for "handshake_complete" and "connection_established" events
6. Verify Connection Health shows "Encrypted tunnel active"

**File Transfer:**
1. On either tab (Listener or Dialer), click "Send File Securely"
2. Select a small text file (e.g., a few KB)
3. Observe file transfer state:
   - State transitions: preparing → in_progress → completed
   - Progress indicator updates
   - File name and size displayed
4. After completion, verify:
   - File transfer status shows "sent successfully"
   - Transfer metrics displayed (file size, duration, approximate throughput)
   - Test Summary shows "Last File Transfer" with completion status
   - Debug Console shows file transfer log entries

**Expected Results:**

- File transfer completes successfully
- File transfer state machine transitions correctly
- Transfer metrics (size, duration, throughput) are displayed
- Test Summary updates with last transfer information
- Debug Console shows file transfer events without errors
- No UI hangs or unhandled exceptions

### Medium-Size File Test

This test validates file transfer with larger files to check for timeouts, partial transfers, and error handling.

**Test Steps:**

1. Establish Listener/Dialer connection (same as above)
2. Select a medium-size file (1-5 MB)
3. Initiate file transfer
4. Monitor:
   - Progress indicator updates smoothly
   - No timeout errors
   - Transfer completes fully
   - File size matches original
   - Transfer duration and throughput are reasonable

**Expected Results:**

- File transfer completes without timeouts
- No partial transfer errors
- Transfer metrics are accurate
- Debug Console shows consistent file transfer log events
- No memory or performance issues in the UI

### File Transfer State Machine Validation

**Test Steps:**

1. Establish connection
2. Select a file
3. Observe state transitions:
   - Initial: "idle"
   - After file selection: "preparing"
   - During transfer: "in_progress"
   - On success: "completed"
   - On error: "failed"
4. Verify UI reflects each state:
   - Button disabled during "in_progress"
   - Status message updates appropriately
   - Progress bar visible during transfer
   - Completion metrics shown on success

**Expected Results:**

- All state transitions occur correctly
- UI reflects current state accurately
- No state machine errors or stuck states

## Verification Criteria

### File Transfer Completion

- File transfer status shows "sent successfully" or equivalent success message
- File transfer state is "completed"
- Transfer metrics (size, duration, throughput) are displayed
- No error messages in Debug Console related to file transfer

### Debug Console Logs

- Debug Console shows file transfer log entries
- Log entries are consistent and without errors
- File transfer events are properly categorized (INFO level)
- No ERROR-level entries related to file transfer

### Test Summary Updates

- Test Summary "Last File Transfer" field updates after transfer
- Transfer status is accurate (Completed/Failed/In progress)
- Transfer metrics are displayed when available
- Summary shows "N/A" when no transfer has occurred

### UI Behavior

- File transfer UI is responsive during transfer
- Progress indicator updates smoothly
- No UI hangs or unhandled exceptions
- Error messages are clear and actionable

## Troubleshooting

### Common Issues

**Dialer started before Listener:**
- Error: Connection timeout or handshake failure
- Solution: Start Listener first, wait for "listener_ready", then start Dialer

**Wrong port or peer address:**
- Error: Connection refused or timeout
- Solution: Verify port and peer address match between Listener and Dialer
- For local testing, use 127.0.0.1 (not 0.0.0.0)

**File transfer fails immediately:**
- Error: "Not connected to peer"
- Solution: Ensure connection is established (Connection Health shows "Encrypted tunnel active") before attempting file transfer

**Transfer hangs or times out:**
- Check Debug Console for error messages
- Verify connection is still active (Connection Health panel)
- Try with a smaller file first
- Check network connectivity between peers

**Transfer metrics not displayed:**
- Verify transfer completed successfully (state is "completed")
- Check that start and end times were recorded
- Metrics may not be available if transfer failed or was interrupted

### Using Debug Tools

**Connection Health Panel:**
- Verify "Encrypted tunnel active" status before file transfer
- Check last handshake and ping latency to confirm connection health

**Debug Console Filters:**
- Use "All" filter to see all file transfer events
- Use "Errors" filter to focus on any file transfer errors
- Use "Status" or "Info" filters to see file transfer progress

**Download Logs:**
- Export session logs when reporting file transfer issues
- Logs include file transfer events and error messages
- Include logs in bug reports for file transfer problems

**Test Summary:**
- Check "Last File Transfer" status for quick overview
- Verify transfer metrics are displayed correctly
- Use summary to confirm transfer completion

## Environment Limitations

Some issues are environment-specific and not product bugs:

- **Large file timeouts in restricted environments:** Container or network limitations may cause timeouts for very large files
- **Transfer speed variations:** Network conditions and system resources affect transfer speed; focus on correctness rather than performance
- **Browser storage limits:** Very large files may be limited by browser FileReader API constraints

## Best Practices

1. **Start with small files:** Test with small files (few KB) before attempting larger transfers
2. **Verify connection first:** Ensure Connection Health shows "Encrypted tunnel active" before file transfer
3. **Monitor Debug Console:** Watch for file transfer events and any errors
4. **Check Test Summary:** Use Test Summary to quickly verify transfer completion
5. **Export logs for analysis:** Download logs when investigating issues or reporting bugs
6. **Test both directions:** Test file transfer from both Listener and Dialer sides
7. **Validate metrics:** Verify transfer metrics (size, duration, throughput) are reasonable

## Additional Resources

- `docs/WEB_TESTER_GUIDE.md`: Comprehensive Web Tester usage guide
- `docs/QUICKSTART_CLI.md`: CLI usage for core tunnel testing
- `docs/VPN_TEST_PLAN.md`: VPN and P2P tunnel test procedures
- `README.md`: Project overview and build instructions

