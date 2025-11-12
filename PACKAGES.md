# CrypRQ Package Build Summary

## macOS Package ✅

**Location:** `dist/macos/`

- **App Bundle:** `CrypRQ.app` (8.8MB binary + resources)
- **DMG:** `CrypRQ-1.0.1-macOS.dmg` (4.6MB compressed)

### Build Commands:
```bash
## Build binary
bash scripts/build-macos.sh

## Create app bundle
VERSION=1.0.1 bash macos/scripts/build-app.sh

## Create DMG
VERSION=1.0.1 bash macos/scripts/create-dmg.sh
```

### Next Steps (Optional):
- Sign with Developer ID: `codesign --sign "Developer ID Application: ..." dist/macos/CrypRQ.app`
- Notarize: `xcrun notarytool submit dist/macos/CrypRQ-1.0.1-macOS.dmg --keychain-profile ...`
- Staple: `xcrun stapler staple dist/macos/CrypRQ-1.0.1-macOS.dmg`

## Windows Package (Structure Ready)

**Location:** `windows/`

- **Packaging Structure:** Created
- **AppxManifest.xml:** Configured for version 1.0.1.0
- **Build Script:** `windows/scripts/build-msix.ps1`

### Build Requirements:
- Windows 10/11 with Windows SDK installed
- Rust toolchain with `x86_64-pc-windows-msvc` target
- PowerShell execution policy allowing scripts

### Build Commands (on Windows):
```powershell
## Cross-compile from macOS (or build on Windows)
cargo build --release --target x86_64-pc-windows-msvc -p cryprq

## On Windows, create MSIX:
cd windows/scripts
.\build-msix.ps1 -Version "1.0.1.0"
```

### Output:
- MSIX will be created at: `dist/windows/CrypRQ_1.0.1.0.msix`

### Note:
Windows cross-compilation from macOS requires Windows SDK libraries. For full MSIX creation, use GitHub Actions Windows runner or a Windows machine.

## Package Checksums

Run these to generate checksums:
```bash
## macOS DMG
shasum -a 256 dist/macos/CrypRQ-1.0.1-macOS.dmg
shasum -a 1 dist/macos/CrypRQ-1.0.1-macOS.dmg

## Windows MSIX (when available)
shasum -a 256 dist/windows/CrypRQ_1.0.1.0.msix
shasum -a 1 dist/windows/CrypRQ_1.0.1.0.msix
```

