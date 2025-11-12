# macOS Network Extension Setup Guide

This guide explains how to set up and build the macOS Network Extension (NEPacketTunnelProvider) for CrypRQ.

## Overview

The Network Extension enables system-wide VPN routing on macOS by:
1. Creating a virtual network interface (TUN)
2. Capturing all system traffic
3. Routing it through the CrypRQ encrypted tunnel
4. Writing decrypted packets back to the TUN interface

## Requirements

- macOS 13.0+ (Ventura or later)
- Xcode 14.0+
- Apple Developer Account (for code signing)
- Network Extension entitlement

## Project Structure

```
apple/
 Sources/
    CrypRQTunnelKit/
        PacketTunnelProvider.swift    # NEPacketTunnelProvider implementation
        CrypRQPacketPump.swift        # Packet forwarding logic
        CrypRQTunnelController.swift  # Tunnel connection management
        PacketPump.swift             # Packet pump protocol
        CrypRQFFI.swift              # Rust FFI bridge (to be implemented)
        CrypRQModels.swift           # Configuration models
        CrypRQError.swift            # Error types
 Package.swift                         # Swift Package Manager manifest
```

## Xcode Project Setup

### 1. Create Xcode Project

1. Open Xcode
2. Create a new project:
   - **Product Name**: CrypRQ
   - **Organization Identifier**: dev.cryprq
   - **Bundle Identifier**: dev.cryprq.CrypRQ
   - **Language**: Swift
   - **Platform**: macOS

### 2. Add Network Extension Target

1. In Xcode, go to **File > New > Target**
2. Select **Network Extension** template
3. Configure:
   - **Product Name**: CrypRQPacketTunnel
   - **Bundle Identifier**: dev.cryprq.CrypRQ.PacketTunnel
   - **Language**: Swift

### 3. Configure Capabilities

1. Select the main app target
2. Go to **Signing & Capabilities**
3. Add **Network Extensions** capability
4. Add **Personal VPN** entitlement

### 4. Configure Network Extension Target

1. Select the PacketTunnel extension target
2. Go to **Signing & Capabilities**
3. Ensure **Network Extensions** capability is enabled
4. Set **App Group** (optional, for app-extension communication)

### 5. Add Swift Package Files

Copy the Swift source files from `apple/Sources/CrypRQTunnelKit/` to your Xcode project:

- `PacketTunnelProvider.swift` → Network Extension target
- `CrypRQPacketPump.swift` → Network Extension target
- `CrypRQTunnelController.swift` → Both targets (shared)
- `PacketPump.swift` → Both targets (shared)
- `CrypRQModels.swift` → Both targets (shared)
- `CrypRQError.swift` → Both targets (shared)
- `CrypRQFFI.swift` → Network Extension target

### 6. Update PacketTunnelProvider.swift

Replace the default `PacketTunnelProvider` class with:

```swift
import NetworkExtension

class PacketTunnelProvider: CrypRQPacketTunnelProvider {
    // Inherits all functionality from CrypRQPacketTunnelProvider
}
```

## Rust FFI Integration

The Network Extension needs to call into the Rust `cryprq` binary for:
- Establishing encrypted tunnel connections
- Sending/receiving encrypted packets
- Key rotation management

### Option 1: Dynamic Library (Recommended)

1. Build Rust as a dynamic library:
   ```bash
   cargo build --release --lib -p cryprq-core
   ```

2. Create C header file using `cbindgen`:
   ```bash
   cbindgen --config cbindgen.toml --crate cryprq-core --output apple/Headers/cryprq.h
   ```

3. Add header to Xcode project

4. Link dynamic library in Xcode:
   - **Build Phases > Link Binary With Libraries**
   - Add `libcryprq_core.dylib`

### Option 2: XPC Service

Create an XPC service that runs the Rust binary and communicates via XPC messages.

## Configuration

The Network Extension receives configuration from the main app via `NETunnelProviderProtocol`:

```swift
let config = TunnelConfiguration(
    peerAddress: "/ip4/127.0.0.1/udp/9999/quic-v1",
    localAddress: "10.0.0.1",
    subnetMask: "255.255.255.0",
    mtu: 1420,
    dnsServers: ["1.1.1.1", "8.8.8.8"]
)

let providerProtocol = NETunnelProviderProtocol()
providerProtocol.providerConfiguration = [
    "config": try JSONEncoder().encode(config)
]
```

## Building

### From Xcode

1. Select the **CrypRQ** scheme
2. Build (Cmd+B)
3. Run (Cmd+R)

### From Command Line

```bash
# Build Rust library
cargo build --release --lib -p cryprq-core

# Build Swift package
cd apple
swift build

# Build Xcode project
xcodebuild -project CrypRQ.xcodeproj \
           -scheme CrypRQ \
           -configuration Release \
           -derivedDataPath build
```

## Code Signing

1. In Xcode, select your Apple Developer team
2. Ensure both app and extension targets are signed
3. For distribution, create provisioning profiles:
   - App ID: `dev.cryprq.CrypRQ`
   - Extension ID: `dev.cryprq.CrypRQ.PacketTunnel`
   - Enable **Network Extensions** capability

## Testing

1. Build and run the app
2. The app should appear in **System Settings > Network**
3. Enable the VPN connection
4. Verify traffic is routed through the tunnel:
   ```bash
   # Check routing table
   netstat -rn | grep utun
   
   # Test connectivity
   curl https://ifconfig.me
   ```

## Troubleshooting

### Extension Not Loading

- Check Console.app for errors
- Verify code signing is correct
- Ensure Network Extension entitlement is present

### Packets Not Forwarding

- Verify TUN interface is created: `ifconfig | grep utun`
- Check routing table: `netstat -rn`
- Review packet pump logs in Console.app

### Connection Timeout

- Ensure Rust binary is accessible
- Check firewall settings
- Verify peer address is correct

## Next Steps

1. Implement Rust FFI bridge (`CrypRQFFI.swift`)
2. Integrate packet forwarding with `node::Tunnel`
3. Add key rotation handling
4. Implement reconnection logic
5. Add status reporting to main app

## References

- [Apple Network Extension Documentation](https://developer.apple.com/documentation/networkextension)
- [NEPacketTunnelProvider](https://developer.apple.com/documentation/networkextension/nepackettunnelprovider)
- [Creating a Network Extension](https://developer.apple.com/documentation/networkextension/creating-a-network-extension)

