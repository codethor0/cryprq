# Icon Coverage

CrypRQ application icon is normalized across all deliverables and platforms.

## Master Icon

**Source**: `store-assets/icon_master_1024.png`

This is the single source of truth for all platform-specific icon generation.

## Icon Generation

Run the master icon generation script:

```bash
bash scripts/generate-icons.sh
```

This generates all platform-specific icons from the master source.

## Platform Coverage

| Platform | Status | Icon Location | Manifest Reference |
|----------|--------|---------------|-------------------|
| **Android** | ✅ Complete | `android/app/src/main/res/mipmap-*/ic_launcher.png` | `AndroidManifest.xml` → `@mipmap/ic_launcher` |
| **iOS** | ✅ Complete | `apple/Sources/CrypRQ/Assets.xcassets/AppIcon.appiconset/` | `Contents.json` + Xcode project |
| **macOS** | ✅ Complete | `branding/CrypRQ.icns` + Assets.xcassets | `Info.plist` → `CFBundleIconFile` |
| **Windows** | ✅ Complete | `windows/Assets/AppIcon.ico` | `AppxManifest.xml` → VisualAssets |
| **Linux Desktop** | ✅ Complete | `packaging/linux/hicolor/*/apps/cryprq.png` | `cryprq.desktop` → `Icon=cryprq` |
| **Electron GUI** | ✅ Complete | `gui/build/icon.{icns,ico,png}` | `package.json` + `electron-builder.yml` |
| **Docker** | ✅ Complete | OCI label `org.opencontainers.image.logo` | Dockerfile LABEL |

## Generated Icon Sizes

### Android
- xxxhdpi: 192×192
- xxhdpi: 144×144
- xhdpi: 96×96
- hdpi: 72×72
- mdpi: 48×48
- Play Store: 512×512

### iOS
- 20pt @2x/@3x (40×40, 60×60)
- 29pt @2x/@3x (58×58, 87×87)
- 40pt @2x/@3x (80×80, 120×120)
- 60pt @2x/@3x (120×120, 180×180)
- 76pt @2x (152×152)
- 83.5pt @2x (167×167)
- App Store: 1024×1024

### macOS
- .icns file with sizes: 16, 32, 128, 256, 512, 1024 (@1x and @2x)

### Windows
- Multi-size .ico: 16, 24, 32, 48, 64, 128, 256

### Linux Desktop
- hicolor theme: 16, 24, 32, 48, 64, 128, 256, 512
- AppImage .DirIcon: 512×512

### Electron GUI
- macOS: `build/icon.icns`
- Windows: `build/icon.ico`
- Linux: `build/icon.png` (512×512)

## Verification

### Android
```bash
aapt dump badging <apk> | grep application-icon
```

### iOS/macOS
```bash
## Check Contents.json
cat apple/Sources/CrypRQ/Assets.xcassets/AppIcon.appiconset/Contents.json

## Verify .icns
file branding/CrypRQ.icns
```

### Windows
```bash
## Check .ico file
file windows/Assets/AppIcon.ico

## Verify manifest references
grep -i "logo\|icon" windows/packaging/AppxManifest.xml
```

### Linux
```bash
## Verify hicolor structure
ls -R packaging/linux/hicolor/

## Check desktop entry
grep Icon= packaging/linux/cryprq.desktop
```

### Electron
```bash
## Verify icon files exist
ls -lh gui/build/icon.*

## Check package.json references
grep -A 5 "icon" gui/package.json
```

### Docker
```bash
## Check OCI label
docker inspect <image> | grep -i logo
```

## CI/CD Integration

Icons are automatically generated and verified in CI:

- **Workflow**: `.github/workflows/icon-enforcement.yml` - Enforces icon coverage on all pushes/PRs/releases
- **Workflow**: `.github/workflows/icons.yml` - Dedicated icon generation workflow
- **Trigger**: On push to main, PR, release tags, or manual dispatch
- **Validation**: **FAILS CI** if required icons are missing or manifests are incorrect
- **Release Blocking**: Releases are blocked if icon verification fails
- **Artifacts**: Uploads `icon_report.txt` and `verify_report.txt` to workflow artifacts and releases

## Icon Report

After generation, check `artifacts/icons/icon_report.txt` for:
- List of all generated icons
- File sizes and checksums
- Platform coverage status

## Rebuilding Packages

After generating icons, rebuild platform packages:

```bash
## Android
cd android && ./gradlew assembleRelease

## iOS/macOS
xcodebuild -workspace <workspace> -scheme <scheme> -configuration Release

## Windows
## Build MSIX with updated AppIcon.ico

## Linux
## Build DEB/RPM/AppImage with hicolor icons

## Electron GUI
cd gui && npm run build
```

## Maintenance

- **Update master icon**: Replace `store-assets/icon_master_1024.png`
- **Regenerate all**: Run `bash scripts/generate-icons.sh`
- **Verify**: Check `artifacts/icons/icon_report.txt`
- **Rebuild**: Rebuild affected platform packages

