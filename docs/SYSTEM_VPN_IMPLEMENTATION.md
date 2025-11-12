# System-Wide VPN Implementation Plan

## Overview

This document outlines the implementation plan for routing all system traffic through the CrypRQ encrypted tunnel.

## Current Status

- ✅ **Control Plane**: Encrypted peer-to-peer connections working (ML-KEM + X25519)
- ✅ **Tunnel Encryption**: ChaCha20-Poly1305 AEAD encryption implemented
- ⚠️ **Data Plane**: Packet forwarding is experimental/incomplete
- ❌ **System Routing**: Not yet implemented

## Architecture

### macOS Implementation

For macOS, we have two options:

1. **Network Extension (Recommended for Production)**
   - Requires Xcode project setup
   - Uses `NEPacketTunnelProvider`
   - Proper system integration
   - Requires code signing and entitlements
   - See `docs/apple.md` for details

2. **TUN Interface (Simpler, for Development)**
   - Direct TUN interface creation using `tun` crate
   - Manual routing configuration
   - Works without Xcode
   - Requires root/admin privileges
   - Good for testing and development

### Implementation Steps

1. **Add TUN Interface Support**
   - Create TUN interface module
   - Implement packet I/O loop
   - Bridge TUN packets to encrypted tunnel

2. **Packet Forwarding**
   - Read packets from TUN interface
   - Encrypt and send via tunnel
   - Receive encrypted packets
   - Decrypt and write to TUN interface

3. **Routing Configuration**
   - Configure TUN interface IP address
   - Set up routing rules
   - Handle DNS resolution

4. **Integration**
   - Add CLI flags for VPN mode
   - Start/stop VPN functionality
   - Status reporting

## Next Steps

1. Add `tun` crate dependency
2. Create `tunnel` module for TUN interface management
3. Implement packet forwarding loop
4. Add routing configuration helpers
5. Integrate with existing CLI

## Security Considerations

- TUN interface requires elevated privileges
- All traffic will be encrypted through the tunnel
- DNS queries should also route through tunnel
- Need to handle local traffic (split tunneling)

## Testing

- Verify all system traffic routes through tunnel
- Test DNS resolution through tunnel
- Verify encryption is working
- Test connection/disconnection
- Measure performance impact

