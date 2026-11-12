# CrypRQ CLI Quickstart

## Overview

CrypRQ provides a P2P encrypted tunnel using QUIC transport with ML-KEM (Kyber768) + X25519 hybrid post-quantum cryptography. This document covers using the CrypRQ CLI for establishing encrypted peer-to-peer connections.

This guide focuses on P2P tunnel functionality. For system-wide VPN behavior (which requires OS-level integration and admin privileges), see `docs/SYSTEM_VPN_IMPLEMENTATION.md` and `docs/VPN_TEST_PLAN.md`.

## Requirements

- CrypRQ binary built and installed (or available in PATH)
- Two hosts or two terminal sessions (for listener/dialer testing)
- Basic understanding of IP addresses and UDP ports
- Network connectivity between the two endpoints

## Examples

### Local Loopback Test (Single Machine)

This test runs both listener and dialer on the same machine using the loopback interface.

**Terminal 1 - Listener:**

```bash
cryprq --listen /ip4/0.0.0.0/udp/10000/quic-v1
```

The listener will:
- Bind to port 10000 on all interfaces (0.0.0.0)
- Generate encryption keys (ML-KEM + X25519)
- Wait for incoming connections
- Display peer ID and connection status

**Terminal 2 - Dialer:**

```bash
cryprq --peer /ip4/127.0.0.1/udp/10000/quic-v1
```

The dialer will:
- Connect to the listener at 127.0.0.1:10000
- Perform QUIC handshake with hybrid encryption
- Establish encrypted tunnel
- Display connection status and peer information

**Expected Output:**

Both terminals should show:
- Local peer ID (libp2p peer identifier)
- Handshake completion events
- Connection established events
- Key rotation events (every 300 seconds by default)
- Ping events with latency measurements

### Two-Host P2P Tunnel

This example connects two separate hosts over a network.

**Host A (Listener):**

```bash
cryprq --listen /ip4/<HOST_A_IP>/udp/10000/quic-v1
```

Replace `<HOST_A_IP>` with the actual IP address of Host A (e.g., `192.168.1.100`).

**Host B (Dialer):**

```bash
cryprq --peer /ip4/<HOST_A_IP>/udp/10000/quic-v1
```

Replace `<HOST_A_IP>` with the same IP address used in the listener command.

**Network Considerations:**

- Ensure firewall rules allow UDP traffic on port 10000 (or your chosen port)
- Both hosts must be able to reach each other's IP addresses
- NAT traversal may be required for hosts behind routers/firewalls
- For production use, consider using a relay or STUN/TURN service for NAT traversal

### VPN Mode Note

The `--vpn` flag enables system-wide routing mode, which attempts to create a TUN interface and route all system traffic through the encrypted tunnel.

**Important:**

- VPN mode requires administrator/root privileges
- TUN device support is required (varies by OS)
- Behavior and capabilities differ significantly between operating systems
- In Docker/container environments, VPN mode typically fails gracefully with clear error messages

**Example with VPN Mode:**

```bash
# Listener with VPN mode
cryprq --listen /ip4/0.0.0.0/udp/10000/quic-v1 --vpn

# Dialer with VPN mode
cryprq --peer /ip4/127.0.0.1/udp/10000/quic-v1 --vpn
```

**Expected Behavior:**

- If TUN device and privileges are available: System-wide routing is enabled
- If TUN device or privileges are missing: Process exits with clear error message, P2P tunnel functionality remains available without `--vpn`

For detailed VPN testing procedures and OS-level integration requirements, see:
- `docs/SYSTEM_VPN_IMPLEMENTATION.md`
- `docs/VPN_TEST_PLAN.md`

## Safety and Limitations

### Encryption

- All traffic between peers is encrypted using ML-KEM (Kyber768) + X25519 hybrid encryption
- Keys are rotated automatically (default interval: 300 seconds)
- Encryption is always enabled; there is no unencrypted mode

### System Routing

- System-wide routing, firewall rules, and DNS behavior are environment-specific
- CrypRQ does not modify system firewall rules or DNS settings automatically
- Users are responsible for configuring their network environment appropriately

### Limitations

- CrypRQ is not a fully audited consumer VPN product
- Users should test thoroughly before relying on it for production use
- NAT traversal may require additional configuration or relay services
- System-wide VPN mode requires OS-specific integration and may not be available in all environments

### Testing Recommendations

- Start with local loopback tests to verify basic functionality
- Test on isolated networks before using in production environments
- Monitor connection health and key rotation events
- Review logs for any unexpected errors or warnings
- For VPN mode, test in a controlled environment with proper privileges and TUN support

## Troubleshooting

### Connection Failures

- **Handshake timeout**: Verify listener is running and reachable
- **Address already in use**: Port is occupied; choose a different port
- **Connection refused**: Firewall may be blocking UDP traffic
- **DNS resolution errors**: Use IP addresses directly instead of hostnames

### VPN Mode Errors

- **"Failed to create TUN device"**: Requires root/admin privileges and TUN support
- **"No such file or directory"**: TUN device not available in this environment
- **System routing not working**: Verify TUN interface was created and routes are configured

### Log Analysis

- Enable verbose logging with `RUST_LOG=info` or `RUST_LOG=debug`
- Look for `event=handshake_complete` to confirm encryption is active
- Check for `event=key_rotation` to verify key rotation is working
- Review `event=connection_established` to confirm tunnel is active

## Additional Resources

- `docs/VPN_TEST_PLAN.md`: Comprehensive testing procedures
- `docs/SYSTEM_VPN_IMPLEMENTATION.md`: VPN implementation details
- `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md`: Troubleshooting guide and support bundle collection
- `README.md`: Project overview and build instructions

## Support Bundle Collection

For troubleshooting issues, run `./scripts/support_bundle.sh` to automatically collect diagnostic information. See `docs/TROUBLESHOOTING_SUPPORT_BUNDLE.md` for details.

