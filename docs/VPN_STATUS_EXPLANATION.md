# System-Wide VPN Status Explanation

## Current Status

###  What's Working

1. **P2P Encrypted Tunnel**: The encrypted connection between peers is fully functional
   - ML-KEM (Kyber768) + X25519 hybrid encryption 
   - Key rotation every 5 minutes 
   - All traffic between the two `cryprq` processes is encrypted 

2. **Connection Status**: Peers can connect and communicate securely

###  What's Not Working Yet

**System-Wide VPN Routing**: Routing all system/browser traffic through the tunnel is not yet implemented.

## Why System-Wide VPN Isn't Working

On macOS, routing all system traffic requires:

1. **Network Extension Framework** (Recommended)
   - Uses `NEPacketTunnelProvider`
   - Requires Xcode project setup
   - Requires code signing and entitlements
   - Proper system integration
   - See `docs/apple.md` for details

2. **OR Root/Admin + TUN Device Access** (Alternative)
   - Direct TUN interface manipulation
   - Requires sudo/admin privileges
   - More complex to implement
   - Less secure (requires elevated privileges)

## Current Implementation

The current VPN mode implementation:
-  Enables VPN mode flag (`--vpn`)
-  Attempts to create TUN interface
-  Logs VPN mode status
-  **Does NOT** actually route system traffic yet
-  **Does NOT** create functional TUN interface on macOS (requires Network Extension)

## What You're Seeing

When you enable VPN mode:
- The P2P encrypted tunnel **IS** working
- All traffic between the two `cryprq` processes **IS** encrypted
- But system/browser traffic is **NOT** being routed through the tunnel

## Next Steps for Full System-Wide VPN

To implement full system-wide VPN routing:

1. **Implement Network Extension** (macOS)
   - Create Xcode project with Packet Tunnel extension
   - Implement `NEPacketTunnelProvider`
   - Set up packet forwarding loop
   - Configure routing tables

2. **OR Implement TUN Interface** (Linux/Alternative)
   - Create TUN device
   - Implement packet forwarding loop
   - Configure routing tables
   - Handle DNS resolution

3. **Packet Forwarding Loop**
   - Read packets from TUN interface
   - Encrypt and send via P2P tunnel
   - Receive encrypted packets from tunnel
   - Decrypt and write to TUN interface

## Current Workaround

For now, the encrypted P2P tunnel is working. To route system traffic:
- Use a proxy application that routes through the tunnel
- OR wait for Network Extension implementation
- OR use on Linux where TUN interfaces are easier to create

## Summary

**P2P Tunnel**:  Working - All peer-to-peer traffic is encrypted  
**System-Wide VPN**:  Not yet implemented - Requires Network Extension framework on macOS

The foundation is in place, but full system-wide routing requires the Network Extension framework implementation.

