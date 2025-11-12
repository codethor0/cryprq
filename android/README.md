# CrypRQ Android Host

This module provides the Android `VpnService` carrier for CrypRQ. The JNI bridge currently stubs out native calls until the Rust FFI is wired in.

## Prerequisites

- Android Studio Giraffe or newer
- Android SDK 34
- Android NDK r26 or newer (`brew install android-ndk` or via Android Studio).  
  Set `ANDROID_NDK_HOME` (e.g. `export ANDROID_NDK_HOME=/opt/homebrew/share/android-ndk`).
- Rust toolchain 1.83.0 with:
  - `cbindgen` (`cargo install cbindgen`)
  - `cargo-ndk` (`cargo install cargo-ndk --version 3.5.4`)
- Rust targets: `rustup target add aarch64-linux-android x86_64-linux-android`

## Build Steps

```bash
## from repository root
ANDROID_NDK_HOME=/opt/homebrew/share/android-ndk ./android/rust/build-android.sh
cd android
./gradlew assembleDebug
```

- Generated shared libraries land in `android/rust/libs/<abi>/libcryprq_core.so`.
- Headers are written to `android/rust/include/cryprq_core.h` for JNI compilation.

## Tests

```bash
./gradlew test
./gradlew connectedCheck  # requires emulator or device
```

## Notable Directories

- `app/src/main/java/dev/cryprq/tunnel/` – VPN service, controller, notifications.
- `app/src/main/java/dev/cryprq/tunnel/jni/` – placeholder JNI bridge.
- `rust/` – helper scripts for cargo + cbindgen builds.

