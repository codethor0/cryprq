# Plan for Addressing Other Builds

## Overview

The web version of CrypRQ is now production-ready with full VPN mode and file transfer support. This document outlines the plan for implementing these features in the remaining builds (GUI, Mobile, CLI).

## Current Status

### ✅ Web Version
- **Status**: Production Ready
- **Features**: VPN mode, file transfer, real-time updates
- **Location**: `web/` directory
- **Documentation**: `docs/WEB_VERSION_STATUS.md`

### ⏸️ Other Builds (Temporarily Disabled)
- **GUI Builds**: Linux, Windows, macOS (disabled in `gui-ci.yml`)
- **Mobile Builds**: Android, iOS (disabled in `mobile-release.yml`)
- **Release Workflows**: Temporarily disabled until builds are fixed

## Implementation Plan

### Phase 1: GUI Builds

#### 1.1 Linux GUI
- [ ] Implement VPN mode toggle in GUI
- [ ] Add file transfer UI components
- [ ] Integrate with CrypRQ binary (similar to web version)
- [ ] Test VPN mode with/without admin privileges
- [ ] Verify file transfer through encrypted tunnel
- [ ] Update GUI documentation

#### 1.2 Windows GUI
- [ ] Implement VPN mode toggle in GUI
- [ ] Add file transfer UI components
- [ ] Handle Windows TAP adapter creation
- [ ] Test VPN mode with/without admin privileges
- [ ] Verify file transfer through encrypted tunnel
- [ ] Update GUI documentation

#### 1.3 macOS GUI
- [ ] Implement VPN mode toggle in GUI
- [ ] Add file transfer UI components
- [ ] Integrate Network Extension framework (if available)
- [ ] Handle TUN interface creation
- [ ] Test VPN mode with/without admin privileges
- [ ] Verify file transfer through encrypted tunnel
- [ ] Update GUI documentation

### Phase 2: Mobile Builds

#### 2.1 Android
- [ ] Implement VPN mode toggle in mobile UI
- [ ] Add file transfer UI components
- [ ] Integrate Android VPN Service
- [ ] Handle Android VPN permissions
- [ ] Test VPN mode functionality
- [ ] Verify file transfer through encrypted tunnel
- [ ] Update mobile documentation

#### 2.2 iOS
- [ ] Implement VPN mode toggle in mobile UI
- [ ] Add file transfer UI components
- [ ] Integrate Network Extension framework
- [ ] Handle iOS VPN permissions
- [ ] Test VPN mode functionality
- [ ] Verify file transfer through encrypted tunnel
- [ ] Update mobile documentation

### Phase 3: CLI Builds

#### 3.1 CLI Enhancements
- [ ] Add VPN mode flag (`--vpn`)
- [ ] Add file transfer commands
- [ ] Improve CLI output and status messages
- [ ] Add encryption status indicators
- [ ] Test all CLI functionality
- [ ] Update CLI documentation

## Implementation Details

### VPN Mode Implementation

All builds should follow the same pattern as the web version:

1. **Toggle/Flag**: Add UI toggle or CLI flag for VPN mode
2. **Privilege Handling**: Gracefully handle missing admin privileges
3. **P2P Fallback**: Keep P2P encrypted tunnel active even if VPN fails
4. **Status Updates**: Provide clear status messages about VPN state

### File Transfer Implementation

All builds should implement:

1. **UI Components**: File selection and transfer UI (for GUI/mobile)
2. **CLI Commands**: File transfer commands (for CLI)
3. **Encryption**: All transfers through ML-KEM + X25519 tunnel
4. **Status**: Real-time transfer status and progress

### Testing Requirements

Each build must pass:

1. **Unit Tests**: All existing tests pass
2. **Integration Tests**: VPN mode and file transfer work correctly
3. **Privilege Tests**: Graceful handling of missing privileges
4. **Encryption Tests**: Verify ML-KEM + X25519 encryption active
5. **File Transfer Tests**: Verify secure file transfer

## Timeline

### Estimated Timeline

- **Phase 1 (GUI Builds)**: 2-3 weeks
- **Phase 2 (Mobile Builds)**: 3-4 weeks
- **Phase 3 (CLI Enhancements)**: 1-2 weeks

**Total Estimated Time**: 6-9 weeks

## Dependencies

### Required
- CrypRQ binary with VPN and file transfer support (✅ Complete)
- Web version implementation as reference (✅ Complete)
- Testing infrastructure (✅ Complete)

### Platform-Specific
- **Linux**: TUN/TAP interface support
- **Windows**: TAP adapter support
- **macOS**: Network Extension framework (optional)
- **Android**: VPN Service API
- **iOS**: Network Extension framework

## Success Criteria

Each build is considered complete when:

1. ✅ VPN mode toggle/flag implemented
2. ✅ File transfer functionality working
3. ✅ All tests passing
4. ✅ Documentation updated
5. ✅ Graceful error handling for privileges
6. ✅ Encryption verified and active

## GitHub Issues

### Issue 1: Implement VPN Mode and File Transfer in GUI Builds
**Labels**: `enhancement`, `gui`, `vpn`, `file-transfer`
**Priority**: High
**Description**: Implement system-wide VPN mode and file transfer in Linux, Windows, and macOS GUI builds. Follow the web version implementation as reference.

### Issue 2: Implement VPN Mode and File Transfer in Mobile Builds
**Labels**: `enhancement`, `mobile`, `vpn`, `file-transfer`
**Priority**: High
**Description**: Implement system-wide VPN mode and file transfer in Android and iOS mobile builds. Integrate platform-specific VPN APIs.

### Issue 3: Enhance CLI with VPN Mode and File Transfer
**Labels**: `enhancement`, `cli`, `vpn`, `file-transfer`
**Priority**: Medium
**Description**: Add VPN mode flag and file transfer commands to CLI. Improve status output and encryption indicators.

## Notes

- The web version implementation serves as the reference for all other builds
- All builds should maintain feature parity with the web version
- Platform-specific VPN APIs may require additional permissions/certificates
- File transfer implementation may vary by platform (UI vs CLI)
- Testing should be comprehensive and cover all edge cases

## Next Steps

1. ✅ Web version finalized and committed
2. ⏳ Create GitHub issues for remaining builds
3. ⏳ Begin Phase 1: GUI Builds implementation
4. ⏳ Test and verify each build
5. ⏳ Re-enable workflows after builds are fixed

