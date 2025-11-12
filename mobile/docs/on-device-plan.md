# On-Device Mode Implementation Plan

## Overview

On-Device mode allows CrypRQ Mobile to run the CrypRQ core directly on the device, eliminating the need for a separate node. This document outlines the implementation plan, risks, and milestones.

## Architecture

### Android

**Approach:**
- Package CrypRQ core (Rust) via JNI (NDK)
- Produce AAR (Android Archive) library
- Wrap start/stop/status in a native module callable from React Native
- Use `VpnService` for tunneling if needed
- Request `android.permission.BIND_VPN_SERVICE` (system-managed UI)

**Implementation Steps:**
1. Create Android NDK module (`android/app/src/main/jni/`)
2. Build Rust core as `.so` library
3. Create JNI bindings (`startNode`, `stopNode`, `getStatus`)
4. Create React Native native module wrapper
5. Integrate `VpnService` for network tunneling
6. Handle foreground service requirements

**Dependencies:**
- Android NDK
- Rust toolchain
- `jni-rs` crate
- React Native native modules

### iOS

**Approach:**
- Use Network Extension (`NEPacketTunnelProvider`)
- Separate Extension target with limited APIs
- IPC to container app
- Requires specific entitlements and Apple review considerations

**Implementation Steps:**
1. Create Network Extension target in Xcode
2. Package Rust core as static library
3. Create Swift wrapper for Rust FFI
4. Implement `NEPacketTunnelProvider` subclass
5. Set up IPC between extension and app
6. Configure entitlements and capabilities

**Dependencies:**
- Xcode with Network Extension capability
- Rust toolchain
- Swift FFI bindings
- App Store Connect API for entitlements

## Risks

### Store Policy Risks
- **Android:** VPN apps require special permissions and may face additional review
- **iOS:** Network Extensions require justification and may be rejected if functionality overlaps with existing VPN apps
- **Both:** Battery/network impact must be clearly communicated

### Technical Risks
- **Battery Impact:** Running VPN core continuously drains battery
- **Network Impact:** Constant key rotation and peer connections consume data
- **Background Limits:** iOS and Android restrict background execution
- **Permissions UX:** Users must grant VPN permissions, which can be confusing

### Security Risks
- **Root/Jailbreak Detection:** Compromised devices pose security risks
- **Foreground Service:** Android requires persistent notification
- **Network Extension Isolation:** iOS extensions run in separate process

## Milestones

### Phase 1: Android POC (Pilot)
- [ ] Build Rust core as `.so` library
- [ ] Create JNI bindings
- [ ] Create React Native native module
- [ ] Basic start/stop/status functionality
- [ ] Surface status in mobile app UI

**Timeline:** 4-6 weeks

### Phase 2: Android VPN Integration
- [ ] Integrate `VpnService`
- [ ] Handle foreground service requirements
- [ ] Test on multiple Android versions
- [ ] Battery/network impact analysis

**Timeline:** 6-8 weeks

### Phase 3: iOS Network Extension
- [ ] Create Network Extension target
- [ ] Package Rust core for iOS
- [ ] Implement `NEPacketTunnelProvider`
- [ ] Set up IPC
- [ ] Submit for Apple review

**Timeline:** 8-10 weeks

### Phase 4: Production Readiness
- [ ] Comprehensive testing
- [ ] Store submission preparation
- [ ] Documentation and user guides
- [ ] Monitoring and crash reporting

**Timeline:** 4-6 weeks

## Current Status

**On-Device mode is currently disabled** in the Settings UI. The toggle shows a warning and cannot be enabled until Phase 1 (Android POC) is complete.

## References

- [Android VpnService Documentation](https://developer.android.com/reference/android/net/VpnService)
- [iOS Network Extension Documentation](https://developer.apple.com/documentation/networkextension)
- [React Native Native Modules](https://reactnative.dev/docs/native-modules-intro)
- [Rust JNI Bindings](https://docs.rs/jni/)

