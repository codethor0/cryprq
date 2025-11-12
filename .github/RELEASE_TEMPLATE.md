# Release Template

## Version: vX.Y.Z

### Changes
<!-- List key changes, features, fixes -->

### Icon Coverage
- **Master**: `store-assets/icon_master_1024.png`
- **Reports**: `artifacts/icons/icon_report.txt`, `artifacts/icons/verify_report.txt`
- **Platforms**: 
  - Android (mdpi..xxxhdpi)
  - iOS/macOS (asset catalog + 1024)
  - Windows (.ico)
  - Linux (hicolor 16..512)
  - GUI (Electron/Tauri)
  - Docker (OCI logo)
- **Result**:  PASS

### Signing Status
<!-- Update based on secrets availability -->
- macOS:  Signed & Notarized /  Unsigned
- Windows:  Signed /  Unsigned
- Android:  Signed /  Unsigned

### Verification
- [ ] Icon verification passed
- [ ] All platform packages built successfully
- [ ] Smoke tests passed
- [ ] Release artifacts uploaded

### Next Steps
- Run smoke tests on each platform
- Monitor for issues post-release

