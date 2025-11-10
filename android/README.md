# CrypRQ Android Host

This module provides the Android `VpnService` carrier for CrypRQ. The JNI bridge currently stubs out native calls until the Rust FFI is wired in.

## Prerequisites

- Android Studio Giraffe or newer
- Android SDK 34, NDK r26d
- Rust toolchain 1.83.0 with `cargo-ndk` and `cbindgen`

## Build Steps

```bash
# from repository root
android/rust/build-android.sh
cd android
./gradlew assembleDebug
```

- Generated shared libraries land in `android/rust/libs/<abi>/`.
- The stub JNI layer logs when the native library is missing; replace with real bindings once `cryp-rq-core` FFI is exposed.

## Tests

```bash
./gradlew test
./gradlew connectedCheck  # requires emulator or device
```

## Notable Directories

- `app/src/main/java/dev/cryprq/tunnel/` – VPN service, controller, notifications.
- `app/src/main/java/dev/cryprq/tunnel/jni/` – placeholder JNI bridge.
- `rust/` – helper scripts for cargo + cbindgen builds.

