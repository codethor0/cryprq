# F-Droid Inclusion Plan

This document outlines the steps to publish the Android VpnService host on F-Droid, including metadata, reproducible build instructions, and pre-submission checks.

## Repository Structure

```
fdroid/
  metadata/dev.cryprq.tunnel.yml
  fastlane/
    metadata/android/en-US/...
  scripts/build.sh
  artifacts/
```

- `metadata/dev.cryprq.tunnel.yml` mirrors F-Droid data fields.
- `fastlane/metadata` houses Play/F-Droid cross-compatible store copy.
- `scripts/build.sh` orchestrates reproducible builds (gradle + cargo-ndk).

## Metadata Template (`dev.cryprq.tunnel.yml`)

```yaml
Categories:
  - Security
License: MIT
WebSite: https://cryprq.dev
SourceCode: https://github.com/codethor0/cryprq
IssueTracker: https://github.com/codethor0/cryprq/issues
Changelog: https://github.com/codethor0/cryprq/releases
Donate: https://cryprq.dev/donate

AutoName: CrypRQ Tunnel
Description: |-
  CrypRQ delivers a post-quantum, zero-trust VPN control plane using a Kyber768 + X25519 hybrid handshake. This build exposes a VpnService tunnel that connects to CrypRQ peers with explicit allowlists and rapid key rotation. No telemetry or analytics.

RepoType: git
Repo: https://github.com/codethor0/cryprq.git

Builds:
  - versionName: 0.1.0-alpha1
    versionCode: 1001
    commit: main
    subdir: android
    gradle:
      - app
    gradleprops:
      - CrypRQ_USE_PQ=1
    output: app/build/outputs/apk/release/app-release.apk
    srclibs:
      - cargo-ndk@2.11.0
    rm:
      - android/.idea
    prebuild:
      - ./rust/build-android.sh
    scanignore:
      - rust/libs/*

AntiFeatures:
  - NonFreeNet
```

- `NonFreeNet` flagged if remote peers are user-controlled (not FOSS servers). Remove if unnecessary.
- Update version fields per release.

## Build Script (`fdroid/scripts/build.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$HOME/Android/Sdk/ndk/26.1.10909125}"

pushd rust
./build-android.sh
popd

./gradlew \
  --no-daemon \
  --project-dir android \
  assembleRelease
```

- Ensure `gradlew` uses the exact wrapper committed in repo.
- Provide default NDK path for reproducibility; F-Droid uses their own environment.

## Reproducible Build Notes

1. Pin Rust and cargo-ndk versions.
2. Use `SOURCE_DATE_EPOCH` when packaging assets (set to commit timestamp).
3. Avoid embedding build timestamps (`android/app/build.gradle` check for `buildConfigField` usage).
4. Document steps in `android/README.md`.

## Fastlane Metadata Mapping

`fastlane/metadata/android/en-US/short_description.txt`:
> Post-quantum VPN control plane with zero-trust peers.

`full_description.txt`:
> - Hybrid ML-KEM (Kyber768) + X25519 handshake over QUIC  
> - Five-minute key rotation with explicit allowlists  
> - Local metrics/health endpoints; no analytics or trackers  
> - Experimental data-plane (control plane only in this release)  

Screenshots: capture onboarding, prominent disclosure, and active tunnel screen (six 1080x1920 PNGs).

## Pre-Submission Checklist

- [ ] `lint` passes: `./gradlew lintRelease`.
- [ ] `./gradlew test` executes without failures.
- [ ] Reproducible build verified via `fdroidserver build --local-data dev.cryprq.tunnel`.
- [ ] No proprietary dependencies (`gradlew dependencies` and `cargo tree` review).
- [ ] Prominent disclosure shown before enabling VPN.
- [ ] Privacy policy hosted at `https://cryprq.dev/privacy`.
- [ ] `fdroid lint` clean (metadata, screenshots, privacy).
- [ ] README includes F-Droid badge placeholder.

## Submission Process

1. Fork [fdroiddata](https://gitlab.com/fdroid/fdroiddata).
2. Add `metadata/dev.cryprq.tunnel.yml` and assets.
3. Run `fdroid checkupdates dev.cryprq.tunnel` to validate build config.
4. Open Merge Request with build logs attached.
5. Respond to reviewer questions (ensuring deterministic builds and license compliance).

## Maintenance

- Automate release updates using GitHub Actions to bump `versionCode`/`versionName` and open MR drafts (`fdroid-update.yml`).
- Monitor F-Droid scanner output for future anti-features (e.g., non-free dependencies).
- Document pinned versions and update cadence in `docs/operations.md`.

