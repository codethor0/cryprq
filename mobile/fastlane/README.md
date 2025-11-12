# Fastlane

Fastlane configuration for building and deploying CrypRQ Mobile.

## Setup

1. Install Fastlane: `gem install fastlane`
2. Configure credentials (see below)

## Android

### Debug Build
```bash
fastlane android build
```

### Release Build (requires keystore)
```bash
fastlane android release
```

### Beta (Internal Testing)
```bash
fastlane android beta
```

## iOS

### Build
```bash
fastlane ios build
```

### Beta (TestFlight)
```bash
fastlane ios beta
```

## Credentials

Create `fastlane/.env` with:
- `ANDROID_KEYSTORE_PATH` (for release builds)
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS`
- `ANDROID_KEY_PASSWORD`
- `APPLE_ID` (for iOS)
- `APPLE_APP_SPECIFIC_PASSWORD` (for TestFlight)

