# Windows Packaging & Store Plan

This guide documents how to package `cryprq.exe` into an MSIX distribution, automate signing in CI, and prepare Microsoft Store submission assets.

## Directory Layout

```
windows/
  packaging/
    AppxManifest.xml
    VisualAssets/
      Square150x150Logo.png
      Square44x44Logo.png
      SplashScreen.png
    Assets/cryprq-icon.ico
  scripts/
    build-msix.ps1
    sign-msix.ps1
    generate-appinstaller.ps1
  store/
    metadata/
      description.md
      screenshots/
      keywords.txt
      privacy-policy-url.txt
```

- `AppxManifest.xml` defines identity, capabilities, and entry point.
- Visual assets follow Microsoft Store requirements (PNG, 1-bit alpha).
- `build-msix.ps1` uses the Windows App SDK tooling or `MakeAppx.exe`.

## AppxManifest Template

```xml
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
         IgnorableNamespaces="uap rescap">
  <Identity Name="ThorThor.CrypRQ"
            Publisher="CN=Thor Thor, O=Thor Labs"
            Version="0.1.0.0" />
  <Properties>
    <DisplayName>CrypRQ</DisplayName>
    <PublisherDisplayName>Thor Labs</PublisherDisplayName>
    <Logo>VisualAssets\Square150x150Logo.png</Logo>
    <Description>Post-quantum VPN control-plane</Description>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.19041.0" MaxVersionTested="10.0.22621.0" />
  </Dependencies>
  <Resources>
    <Resource Language="en-US" />
  </Resources>
  <Applications>
    <Application Id="CrypRQ"
                 Executable="CrypRQ\cryprq.exe"
                 EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="CrypRQ"
                          Description="Post-quantum VPN control-plane"
                          BackgroundColor="transparent"
                          Square150x150Logo="VisualAssets\Square150x150Logo.png"
                          Square44x44Logo="VisualAssets\Square44x44Logo.png" />
    </Application>
  </Applications>
</Package>
```

## Packaging Script (`build-msix.ps1`)

```powershell
param(
    [string]$Version = "0.1.0.0",
    [string]$Configuration = "Release"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Definition
$repo = Resolve-Path "$root\..\.."

$buildDir = Join-Path $root "build"
Remove-Item $buildDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $buildDir | Out-Null

# Copy payload
Copy-Item "$repo\target\release\cryprq.exe" "$buildDir\CrypRQ" -Force
Copy-Item "$root\..\packaging\AppxManifest.xml" "$buildDir\AppxManifest.xml"
Copy-Item "$root\..\packaging\VisualAssets" "$buildDir\VisualAssets" -Recurse

& "$env:ProgramFiles(x86)\Windows Kits\10\bin\x64\MakeAppx.exe" pack `
    /d $buildDir `
    /p "$root\CrypRQ_$Version.msix" `
    /o
```

## Signing Script (`sign-msix.ps1`)

```powershell
param(
    [string]$MsixPath,
    [string]$CertPath,
    [string]$Password
)

$timestampUrl = "http://timestamp.digicert.com"
& "$env:ProgramFiles(x86)\Windows Kits\10\bin\x64\signtool.exe" sign `
    /fd SHA256 `
    /f $CertPath `
    /p $Password `
    /tr $timestampUrl `
    /td SHA256 `
    $MsixPath
```

During CI we can self-sign with a test certificate.

## App Installer Manifest (`generate-appinstaller.ps1`)

Creates `CrypRQ.appinstaller` for sideload distribution:

```powershell
param(
    [string]$Version,
    [string]$Uri
)

@"
<?xml version="1.0" encoding="utf-8"?>
<AppInstaller xmlns="http://schemas.microsoft.com/appx/appinstaller/2018">
  <MainPackage Name="ThorThor.CrypRQ"
               Version="$Version"
               Publisher="CN=Thor Thor, O=Thor Labs"
               Uri="$Uri"
               ProcessorArchitecture="x64"
               />
  <UpdateSettings>
    <OnLaunch HoursBetweenUpdateChecks="24" />
  </UpdateSettings>
</AppInstaller>
"@ | Out-File -FilePath "CrypRQ.appinstaller" -Encoding utf8
```

## GitHub Actions Workflow (`.github/workflows/windows-msix.yml`)

```yaml
name: Windows MSIX

on:
  workflow_dispatch:
  push:
    branches: [ main ]

jobs:
  msix:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: 1.83.0
      - name: Build cryprq.exe
        run: cargo build --release -p cryprq
      - name: Build MSIX
        run: windows\scripts\build-msix.ps1 -Version ${{ github.run_number }}
      - name: Self-sign package
        run: |
          $cert = New-SelfSignedCertificate -Type CodeSigning -Subject "CN=CrypRQ CI" -KeyAlgorithm RSA -KeyLength 4096 -CertStoreLocation "Cert:\CurrentUser\My"
          $pwd = ConvertTo-SecureString "temporary" -AsPlainText -Force
          Export-PfxCertificate -Cert $cert -FilePath CrypRQ-TestCert.pfx -Password $pwd
          windows\scripts\sign-msix.ps1 -MsixPath windows\CrypRQ_${{ github.run_number }}.msix -CertPath CrypRQ-TestCert.pfx -Password "temporary"
      - name: Generate App Installer
        run: windows\scripts\generate-appinstaller.ps1 -Version ${{ github.run_number }} -Uri "https://example.com/CrypRQ_${{ github.run_number }}.msix"
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: cryprq-msix
          path: |
            windows\CrypRQ_${{ github.run_number }}.msix
            CrypRQ.appinstaller
```

- Replace `example.com` with staging CDN.
- In production, use a real code-signing cert; for Store submission, App Center or Partner Center re-signs.

## Store Metadata Checklist

| Item | Notes |
|------|-------|
| Short description | “Post-quantum VPN control-plane for zero-trust environments.” |
| Long description | Highlight ML-KEM + X25519, reproducible builds, no telemetry. |
| Screenshots | 1920x1080 (desktop app), console logs, configuration screen. |
| Icons | 300x300 and 1000x1000 PNGs. |
| Capabilities | `internetClient`, optionally `runFullTrust`. |
| Privacy policy | Link to `https://cryprq.dev/privacy.html`. |
| Age rating | Typically 3+ / “General”. |
| Release notes | Summaries of changes. |

## Store Signing Considerations

- When submitting to the Microsoft Store, the package is re-signed by Microsoft. Your signature is stripped; only store metadata is required.
- For sideloading, ensure your cert is trusted (corporate root certificate) or instruct users to install the `.cer` file.
- Document signing process in `docs/release_windows.md`.

## Next Steps

1. Create `windows/` directory with packaging assets.
2. Implement PowerShell scripts and Appx manifest.
3. Add GitHub Actions workflow to CI.
4. Produce test MSIX and validate installation on Windows 11.
5. Gather screenshots/copy for Store metadata.

