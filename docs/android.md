# Android Host Integration Plan

This document outlines the architecture and implementation steps for the Android host that wraps the `cryp-rq-core` FFI via JNI and exposes a `VpnService` tunnel. The goal is to produce a shippable module that you can drop into an Android application with minimal wiring.

## Project Layout

```
android/
  build.gradle.kts
  settings.gradle.kts
  gradle/...
  app/
    src/main/
      AndroidManifest.xml
      java/dev/cryprq/app/MainActivity.kt
      java/dev/cryprq/app/settings/SettingsFragment.kt
      java/dev/cryprq/tunnel/CrypRqVpnService.kt
      java/dev/cryprq/tunnel/CrypRqTunnelController.kt
      java/dev/cryprq/tunnel/jni/CrypRqNative.kt
      java/dev/cryprq/tunnel/jni/CrypRqPacketPump.kt
      java/dev/cryprq/tunnel/notifications/ForegroundNotification.kt
      res/layout/activity_main.xml
      res/xml/vpn_run_disclosure.xml
    src/androidTest/java/dev/cryprq/app/**/*
    src/test/java/dev/cryprq/app/**/*
  rust/
    build-android.sh  # cargo + ndk-build script
    include/cryprq_core.h
    libs/arm64-v8a/libcryprq_core.so
    libs/x86_64/libcryprq_core.so
```

- `rust/` houses the compiled `libcryprq_core.so` and the generated JNI header (`cryprq_core.h`).
- The Kotlin package `dev.cryprq.tunnel.*` isolates the JNI + VPN logic.
- Instrumentation tests live under `src/androidTest`.

## JNI Bridge (`CrypRqNative`)

```kotlin
package dev.cryprq.tunnel.jni

import android.util.Log

object CrypRqNative {
    init {
        System.loadLibrary("cryprq_core")
    }

    @JvmStatic external fun init(config: Config): Long
    @JvmStatic external fun connect(handle: Long, params: PeerParams): Int
    @JvmStatic external fun readPacket(handle: Long, buffer: ByteArray): Int
    @JvmStatic external fun writePacket(handle: Long, buffer: ByteArray, len: Int): Int
    @JvmStatic external fun onNetworkChange(handle: Long): Int
    @JvmStatic external fun close(handle: Long)

    data class Config(
        val logLevel: String?,
        val allowPeers: List<String>,
    )

    data class PeerParams(
        val mode: Int, // 0 = listen, 1 = dial
        val multiaddr: String,
    )
}
```

- Use `@Keep` annotations if you enable R8/ProGuard.
- JNI functions translate errors into Kotlin exceptions (e.g. wrap `Int` return codes with a helper).

### Native Glue (`android/src/main/jni/CrypRqNative.cpp`)

```cpp
#include <jni.h>
#include "cryprq_core.h"

extern "C"
JNIEXPORT jlong JNICALL
Java_dev_cryprq_tunnel_jni_CrypRqNative_init(
        JNIEnv *env, jclass, jobject configObj) {
    // Extract fields via JNI GetObjectClass/GetFieldID,
    // construct CrypRqConfig, and call cryprq_init().
}

extern "C"
JNIEXPORT jint JNICALL
Java_dev_cryprq_tunnel_jni_CrypRqNative_connect(
        JNIEnv *env, jclass, jlong handle, jobject paramsObj) {
    // Forward to cryprq_connect().
}

// Additional wrappers for read/write/onNetworkChange/close...
}
```

Build the JNI glue with the Android NDK (Gradle `externalNativeBuild` or cargo-ndk + CMake).

## VpnService (`CrypRqVpnService`)

Key responsibilities:

1. Build the VPN TUN interface (`Builder` API):
   ```kotlin
   val builder = Builder()
       .setSession("CrypRQ")
       .setConfigureIntent(pendingIntent)
       .setMtu(settings.mtu)
   settings.routes.forEach { builder.addRoute(it.address, it.prefixLength) }
   settings.dns.forEach { builder.addDnsServer(it) }
   val iface = builder.establish() ?: throw IllegalStateException("Failed to establish TUN")
   ```

2. Promote service to foreground:
   ```kotlin
   startForeground(NOTIFICATION_ID, ForegroundNotification.build(context))
   ```

3. Initialise Rust handle and launch packet pump:
   ```kotlin
   val nativeHandle = CrypRqNative.init(
       CrypRqNative.Config(logLevel, allowPeers)
   )
   CrypRqNative.connect(
       nativeHandle,
       CrypRqNative.PeerParams(
           mode = CrypRqNative.MODE_LISTENER,
           multiaddr = settings.multiaddr
       )
   )
   CrypRqPacketPump.start(iface.fileDescriptor, nativeHandle)
   ```

4. Handle stop (`onRevoke` / `onDestroy`):
   ```kotlin
   CrypRqPacketPump.stop()
   CrypRqNative.close(nativeHandle)
   stopForeground(STOP_FOREGROUND_REMOVE)
   ```

## Packet Pump

Use coroutines on `Dispatchers.IO` with a `ParcelFileDescriptor.AutoCloseInputStream` and `AutoCloseOutputStream`.

```kotlin
object CrypRqPacketPump {
    fun start(fd: Int, handle: Long) {
        val tunInput = FileInputStream(parcelFd.fileDescriptor)
        val tunOutput = FileOutputStream(parcelFd.fileDescriptor)
        scope.launch {
            val buf = ByteArray(65535)
            while (isActive) {
                val len = tunInput.read(buf)
                if (len > 0) {
                    CrypRqNative.writePacket(handle, buf, len)
                }
            }
        }
        scope.launch {
            val buf = ByteArray(65535)
            while (isActive) {
                val len = CrypRqNative.readPacket(handle, buf)
                if (len > 0) {
                    tunOutput.write(buf, 0, len)
                }
            }
        }
    }
}
```

Gracefully stop coroutines on `close()` or VPN teardown.

## Prominent Disclosure & Settings

- `res/xml/vpn_run_disclosure.xml`:
  ```xml
  <vpn-profile xmlns:android="http://schemas.android.com/apk/res/android"
      android:name="@string/app_name"
      android:description="@string/disclosure_text" />
  ```
- Settings screen includes toggles for:
  - MTU
  - Routes (comma-separated CIDRs)
  - DNS servers
  - Peer multiaddr
  - Allowlisted peer IDs
- Store `SharedPreferences` + `PreferenceFragmentCompat`.

### Foreground Notification

`ForegroundNotification.build(context)` sets:
- Service channel (`IMPORTANCE_LOW`)
- Content text (e.g. “CrypRQ tunnel active”)
- Pending intent to reopen main activity

## Instrumentation Tests

- Use `androidx.test.runner.AndroidJUnitRunner`.
- Tests:
  1. Launch service with mock JNI (inject interface that records calls) to verify config serialization.
  2. Verify `Builder` is invoked with expected MTU/routes/DNS (via Espresso on a fake `VpnService` subclass).
  3. Ensure foreground notification posted on start, removed on stop.
  4. Packet pump loops: use `ParcelFileDescriptor.createPipe()` to simulate TUN I/O.

## Google Play VPN Declaration

**Prominent disclosure copy (in-app):**
> CrypRQ creates a local VPN tunnel to encrypt peer-to-peer control traffic. No traffic is routed to third-party servers. You can choose which peers to connect to, and configuration stays on your device.

**Checklist:**
- [ ] `VPNService` listed in manifest with `<uses-permission android:name="android.permission.BIND_VPN_SERVICE" />`.
- [ ] Foreground notification visible while active.
- [ ] Play Console “VPN service” declaration filled (description, data handling).
- [ ] Privacy policy URL leads to repo `docs/privacy.md`.
- [ ] `prominent disclosure` shown before enabling tunnel.

## Build Script (`rust/build-android.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail
TARGETS=("aarch64-linux-android" "x86_64-linux-android")
for target in "${TARGETS[@]}"; do
  cargo ndk -t "${target}" -o ../android/rust/libs/${target_arch} --release -p cryp-rq-core
done
cbindgen --config ../cbindgen.toml --crate cryprq_core --output ../android/rust/include/cryprq_core.h
```

## Next Steps

1. Scaffold Gradle project (`android/`).
2. Implement JNI glue + load `.so`.
3. Wire `CrypRqVpnService`, `CrypRqTunnelController`, and notification.
4. Add instrumentation tests and CI (GitHub Actions `android.yml` calling `./gradlew connectedCheck`).
5. Document developer onboarding (`README.android.md`) including NDK/cargo prerequisites.

