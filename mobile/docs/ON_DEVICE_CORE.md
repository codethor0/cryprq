# On-Device Core Implementation Plan

This document outlines the plan for embedding CrypRQ core functionality directly on mobile devices, enabling the phone itself to run a CrypRQ node/VPN.

## Overview

The current mobile app operates in "controller mode", connecting to external CrypRQ nodes. This plan extends the app to run CrypRQ core natively on the device, providing a self-contained VPN solution.

## Architecture

### Android

#### Approach: JNI Bridge + VpnService

1. **Native Module**
   - Package `cryp-rq-core` (Rust FFI) as an Android Archive (AAR)
   - Create JNI wrapper in `android/app/src/main/cpp/`
   - Expose functions: `startNode()`, `stopNode()`, `getStatus()`, `readPacket()`, `writePacket()`

2. **VpnService Integration**
   - Extend existing `VpnService` pattern from desktop Android app
   - Request `android.permission.BIND_VPN_SERVICE` (system-managed UI)
   - Create TUN interface for packet I/O
   - Bridge TUN packets to/from Rust core via JNI

3. **React Native Bridge**
   - Create native module (`CrypRQNativeModule.java`)
   - Expose methods to React Native:
     ```typescript
     interface CrypRQNative {
       startNode(config: NodeConfig): Promise<void>;
       stopNode(): Promise<void>;
       getStatus(): Promise<NodeStatus>;
       onPacket(callback: (packet: Uint8Array) => void): void;
       writePacket(packet: Uint8Array): Promise<void>;
     }
     ```

4. **Build Integration**
   - Add Rust toolchain targets: `aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`
   - Use `cargo-ndk` for cross-compilation
   - Link `.so` libraries in `android/app/build.gradle`

### iOS

#### Approach: Network Extension + IPC

1. **Network Extension Target**
   - Create separate `CrypRQPacketTunnel` extension target
   - Use `NEPacketTunnelProvider` for VPN tunneling
   - Limited APIs: no UI, restricted network access
   - Requires App Group for IPC with container app

2. **Core Integration**
   - Build `cryp-rq-core` for iOS targets: `aarch64-apple-ios`, `x86_64-apple-ios-sim`
   - Create Swift FFI wrapper (similar to `apple/Sources/CrypRQTunnelKit/`)
   - Link static library in extension target

3. **IPC Mechanism**
   - Use `App Groups` (`group.io.cryprq.mobile`) for shared data
   - Use `CFNotificationCenter` or `DarwinNotificationCenter` for events
   - Container app sends commands via UserDefaults/FileCoordinator
   - Extension reads commands and updates status

4. **React Native Bridge**
   - Create native module (`CrypRQNativeModule.m`)
   - Bridge to container app's IPC layer
   - Expose same interface as Android

## Implementation Milestones

### Phase 1: Android POC (4-6 weeks)

**Week 1-2: Native Module Setup**
- [ ] Set up Rust build for Android targets
- [ ] Create JNI wrapper functions
- [ ] Build AAR with `cryp-rq-core`
- [ ] Test basic FFI calls from Java

**Week 3-4: VpnService Integration**
- [ ] Create `CrypRQVpnService` extending `VpnService`
- [ ] Implement TUN interface creation
- [ ] Bridge TUN packets to Rust core
- [ ] Test packet I/O loop

**Week 5-6: React Native Bridge**
- [ ] Create `CrypRQNativeModule`
- [ ] Expose start/stop/status methods
- [ ] Test from React Native app
- [ ] End-to-end test: start node, connect peer, verify packets

### Phase 2: iOS POC (6-8 weeks)

**Week 1-2: Network Extension Setup**
- [ ] Create `CrypRQPacketTunnel` extension target
- [ ] Configure App Groups and entitlements
- [ ] Set up basic `NEPacketTunnelProvider` skeleton

**Week 3-4: Core Integration**
- [ ] Build Rust core for iOS targets
- [ ] Create Swift FFI wrapper
- [ ] Link static library in extension

**Week 5-6: IPC Implementation**
- [ ] Implement App Group shared storage
- [ ] Create command/status IPC protocol
- [ ] Test IPC between container and extension

**Week 7-8: React Native Bridge**
- [ ] Create native module
- [ ] Bridge IPC to React Native
- [ ] End-to-end test

### Phase 3: Production Hardening (4-6 weeks)

- [ ] Error handling and recovery
- [ ] Battery optimization
- [ ] Network state management
- [ ] Background execution limits
- [ ] Store submission preparation

## Risks and Considerations

### Store Policy Risks

**Android:**
- VPN apps require prominent disclosure
- Google Play may require additional review
- Must comply with VPN service policies

**iOS:**
- Network Extensions require Apple review
- Entitlements must be justified
- May require enterprise distribution for certain features
- TestFlight beta testing recommended before App Store submission

### Technical Risks

1. **Battery Impact**
   - Continuous packet processing drains battery
   - Mitigation: Adaptive polling, efficient packet handling, background limits

2. **Network Impact**
   - VPN routing affects all network traffic
   - Mitigation: Clear user education, connection status indicators

3. **Permissions UX**
   - VPN permission request is intrusive
   - Mitigation: Clear onboarding, explain benefits

4. **Background Limits**
   - iOS: Strict background execution limits
   - Android: Doze mode and app standby
   - Mitigation: Foreground service (Android), background tasks (iOS)

5. **Memory Constraints**
   - Mobile devices have limited RAM
   - Mitigation: Efficient memory usage, monitor memory pressure

### Security Considerations

1. **Key Storage**
   - Use platform keychains (iOS Keychain, Android Keystore)
   - Never store keys in plaintext

2. **Network Isolation**
   - Ensure VPN traffic is isolated from app traffic
   - Verify TUN interface routing

3. **Code Signing**
   - All native code must be signed
   - Extension must be signed with same team ID

## Testing Strategy

### Unit Tests
- JNI wrapper functions
- IPC protocol
- Native module methods

### Integration Tests
- TUN packet I/O
- Core node lifecycle
- React Native bridge

### E2E Tests
- Start node from app
- Connect to peer
- Verify VPN routing
- Stop node gracefully

## Store Submission Checklist

### Android
- [ ] VPN disclosure screen
- [ ] Privacy policy link
- [ ] Permissions justification
- [ ] Battery usage disclosure
- [ ] Network usage disclosure

### iOS
- [ ] Network Extension justification
- [ ] App Group configuration
- [ ] Entitlements documentation
- [ ] Privacy policy
- [ ] TestFlight beta testing

## Pilot POC Scope

**Android-only initial implementation:**

1. Basic node start/stop
2. Single peer connection
3. TUN packet forwarding
4. Status display in React Native

**Success Criteria:**
- App can start a local CrypRQ node
- Node connects to a peer
- VPN routes traffic through TUN interface
- Status updates appear in React Native UI
- Node stops gracefully

## Future Enhancements

- Multi-peer support
- Advanced routing rules
- Kill switch functionality
- Split tunneling
- Custom DNS

## References

- [Android VpnService Documentation](https://developer.android.com/reference/android/net/VpnService)
- [iOS Network Extension Guide](https://developer.apple.com/documentation/networkextension)
- [React Native Native Modules](https://reactnative.dev/docs/native-modules-intro)
- Existing Android implementation: `android/app/src/main/java/dev/cryprq/tunnel/`
- Existing iOS implementation: `apple/Sources/CrypRQTunnelKit/`

