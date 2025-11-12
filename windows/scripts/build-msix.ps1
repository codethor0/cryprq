# © 2025 Thor Thor
# Contact: codethor@gmail.com
# SPDX-License-Identifier: MIT

param(
    [string]$Version = "1.0.1.0",
    [string]$Configuration = "Release"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Definition
$repo = Resolve-Path "$root\..\.."

$buildDir = Join-Path $root "..\build"
Remove-Item $buildDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $buildDir | Out-Null

# Copy payload
$exePath = "$repo\target\x86_64-pc-windows-msvc\release\cryprq.exe"
if (-not (Test-Path $exePath)) {
    Write-Host "ERROR: Binary not found at $exePath"
    Write-Host "Build with: cargo build --release --target x86_64-pc-windows-msvc -p cryprq"
    exit 1
}

New-Item -ItemType Directory -Path "$buildDir\CrypRQ" | Out-Null
Copy-Item $exePath "$buildDir\CrypRQ\cryprq.exe" -Force

# Copy manifest and assets
Copy-Item "$root\..\packaging\AppxManifest.xml" "$buildDir\AppxManifest.xml"
if (Test-Path "$root\..\packaging\VisualAssets") {
    Copy-Item "$root\..\packaging\VisualAssets" "$buildDir\VisualAssets" -Recurse
}

# Update version in manifest
$manifest = Get-Content "$buildDir\AppxManifest.xml" -Raw
$manifest = $manifest -replace 'Version="[^"]*"', "Version=`"$Version`""
Set-Content "$buildDir\AppxManifest.xml" -Value $manifest

# Find MakeAppx.exe
$makeAppx = Get-ChildItem "$env:ProgramFiles(x86)\Windows Kits\10\bin\*\x64\MakeAppx.exe" -ErrorAction SilentlyContinue | Select-Object -First 1

if (-not $makeAppx) {
    Write-Host "ERROR: MakeAppx.exe not found. Install Windows SDK."
    Write-Host "Package structure created at: $buildDir"
    Write-Host "Run MakeAppx.exe manually to create MSIX."
    exit 1
}

$msixPath = "$root\..\dist\windows\CrypRQ_$Version.msix"
New-Item -ItemType Directory -Path (Split-Path $msixPath) -Force | Out-Null

& $makeAppx.FullName pack `
    /d $buildDir `
    /p $msixPath `
    /o

Write-Host ""
Write-Host "=== MSIX Created ==="
Write-Host "MSIX: $msixPath"
Get-Item $msixPath | Select-Object Name, Length, LastWriteTime

