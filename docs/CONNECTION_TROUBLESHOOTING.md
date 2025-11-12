# Connection Troubleshooting Guide

## Current Status

The connection between listener and dialer is not working. The listener starts successfully but exits immediately ("exit null"), preventing the dialer from connecting.

## Testing Setup

To test the connection, you need **two browser tabs/windows**:

1. **Tab 1 (Listener)**: 
   - Mode: `listener`
   - Port: `9999`
   - Click "Connect"
   - Should see "Listening on /ip4/127.0.0.1/udp/9999/quic-v1"

2. **Tab 2 (Dialer)**:
   - Mode: `dialer`
   - Port: `9999`
   - Peer: `/ip4/127.0.0.1/udp/9999/quic-v1`
   - Click "Connect"
   - Should connect to the listener

## System-Wide VPN Routing

Once connected, for system-wide VPN routing:

1. **P2P Tunnel**: ✅ Working - All traffic between peers is encrypted via libp2p QUIC
2. **System-Wide VPN**: ⚠️ Requires Network Extension framework on macOS
   - The encrypted tunnel between peers is active
   - Routing all system/browser traffic requires macOS Network Extension (NEPacketTunnelProvider)
   - See `docs/NETWORK_EXTENSION_SETUP.md` for implementation details

## Current Issues

1. **Listener Exiting Immediately**: The listener process exits right after starting ("exit null")
   - This prevents the dialer from connecting
   - Need to investigate why the process is exiting

2. **Packet Forwarding Not Integrated**: TUN interface is created but not forwarding packets
   - Need to integrate TUN packet forwarding with p2p connection
   - Need to configure routing tables to route system traffic through TUN

## Next Steps

1. Fix listener exit issue
2. Get listener and dialer connecting successfully
3. Integrate TUN packet forwarding with p2p connection
4. Configure routing tables for system-wide VPN
5. See encryption events in console for browser traffic

