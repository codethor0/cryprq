# Automated Browser Testing for CrypRQ Docker VPN

## Quick Start

Run the complete automated test suite:

```bash
npm run automated-test
```

Or manually:

```bash
./scripts/automated-browser-test.sh
```

## What It Does

The automated test script:

1. **Checks Docker** - Verifies Docker is running, starts it if needed
2. **Starts Container** - Ensures `cryprq-listener` container is running
3. **Builds Web UI** - Compiles React app if needed
4. **Starts Web Server** - Launches web server with Docker mode enabled
5. **Opens Browser** - Automatically opens browser to web UI
6. **Runs Tests** - Executes Playwright tests to verify:
   - Web UI loads correctly
   - Listener connects (Docker mode)
   - Dialer connects to container
   - Connection established (verified via container logs)
   - Encryption status displayed
   - Container logs stream to debug console

## Test Verification

The tests verify:

 **Connection Working** - Container logs show "Inbound connection established"  
 **Encryption Active** - ML-KEM + X25519 hybrid encryption displayed  
 **Docker Mode** - Container IP and Docker mode messages appear  
 **Event Streaming** - Container logs appear in debug console  
 **Status Updates** - Connection status updates correctly  

## Manual Testing

If you want to test manually:

1. **Start everything:**
   ```bash
   ./scripts/automated-browser-test.sh
   ```

2. **In the browser:**
   - Select **listener** mode → Click Connect
   - Open new tab → Select **dialer** mode → Click Connect
   - Watch debug console for connection events

3. **Verify encryption:**
   - Check debug console for encryption/decryption logs
   - Container logs show connection established
   - Status shows "Connected"

## Troubleshooting

### Event Stream Errors

If you see "event stream error" in the console:
- This is normal - EventSource auto-reconnects
- Check that web server is running: `lsof -ti:8787`
- Restart web server if needed

### Container Not Running

```bash
# Check container status
docker ps | grep cryprq-listener

# Start container
./scripts/docker-vpn-start.sh
```

### Web Server Not Starting

```bash
# Check if port is in use
lsof -ti:8787

# Kill and restart
kill $(lsof -ti:8787)
./scripts/automated-browser-test.sh
```

## Next Steps

Once connection is verified:
1. **Encryption is working** - Container handles all encryption
2. **Traffic routing** - Next: Route browser/system traffic through container
3. **Internet routing** - Next: Route all traffic through encrypted tunnel to Internet

The foundation is working - encryption between Mac and container is active!

