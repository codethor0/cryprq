# Apple Host Integration Plan

This document covers the architecture and implementation steps for the Apple ecosystem: a shared Swift package, a macOS host app, and an iOS/macOS Packet Tunnel Provider extension that bridges to `cryp-rq-core`.

## Project Topology

```
apple/
  CrypRQ.xcworkspace
  Shared/
    Package.swift
    Sources/CrypRQTunnelKit/
      CrypRQTunnelController.swift
      CrypRQFFI.swift
      PacketPump.swift
      Configuration.swift
  macOS/
    CrypRQTunnelApp/
      AppDelegate.swift
      TunnelManager.swift
      UI/*
  iOS/
    CrypRQTunnelHost/
      AppDelegate.swift
      UI/*
    CrypRQPacketTunnel/
      PacketTunnelProvider.swift
      Info.plist
      Resources/NetworkExtension.entitlements
  Scripts/
    build-universal.sh
    generate-headers.sh
```

- `CrypRQTunnelKit` (Swift package) wraps the FFI, handles configuration parsing, and exposes a simple API to the host apps/extensions.
- Both macOS and iOS hosts embed the packet tunnel extension and share UI code via SwiftUI or UIKit.

## FFI Bridging (`CrypRQFFI.swift`)

```swift
import Foundation

public enum CrypRQError: Error {
    case nullPointer
    case invalidUtf8
    case invalidArgument
    case alreadyConnected
    case unsupported
    case runtime
    case internalError
}

public final class CrypRQHandle {
    private var raw: UnsafeMutableRawPointer?

    public init(config: Config) throws {
        var ffiConfig = config.toFFI()
        var outHandle: UnsafeMutablePointer<CrypRqHandleOpaque>? = nil
        let code = cryprq_init(&ffiConfig, &outHandle)
        guard code == CRYPRQ_OK, let handle = outHandle else {
            throw CrypRQError(code: code)
        }
        raw = UnsafeMutableRawPointer(handle)
    }

    public func connect(params: PeerParams) throws {
        guard let raw else { throw CrypRQError.nullPointer }
        var ffiParams = params.toFFI()
        let code = cryprq_connect(raw.assumingMemoryBound(to: CrypRqHandleOpaque.self), &ffiParams)
        guard code == CRYPRQ_OK else { throw CrypRQError(code: code) }
    }

    public func close() {
        guard let raw else { return }
        cryprq_close(raw.assumingMemoryBound(to: CrypRqHandleOpaque.self))
        self.raw = nil
    }
}
```

- Use `deinit` to ensure `close()` is called.
- Map `CrypRqErrorCode` enum to Swift `Error`.
- `Config` includes log level, allowlisted peers, etc.
- `PeerParams` carries mode (`listen`/`dial`) and multiaddr string.

### Header Generation Script

`Scripts/generate-headers.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cbindgen --config "${ROOT}/cbindgen.toml" --crate cryprq_core --output "${ROOT}/apple/Shared/include/cryprq_core.h"
```

Add the header to Xcode build settings (`HEADER_SEARCH_PATHS`).

## Packet Tunnel Provider (`PacketTunnelProvider.swift`)

```swift
import NetworkExtension

final class PacketTunnelProvider: NEPacketTunnelProvider {
    private var cryprq: CrypRQHandle?
    private var pump: PacketPump?

    override func startTunnel(options: [String : NSObject]? = nil,
                              completionHandler: @escaping (Error?) -> Void) {
        do {
            let configuration = try TunnelConfiguration(from: options)
            let providerConfig = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "10.0.0.1")
            providerConfig.mtu = NSNumber(value: configuration.mtu)
            providerConfig.ipv4Settings = NEIPv4Settings(addresses: [configuration.address],
                                                         subnetMasks: ["255.255.255.0"])
            providerConfig.ipv4Settings?.includedRoutes = configuration.routes
            providerConfig.dnsSettings = NEDNSSettings(servers: configuration.dns)
            setTunnelNetworkSettings(providerConfig) { [weak self] error in
                guard error == nil else { completionHandler(error); return }
                do {
                    let handle = try CrypRQHandle(config: configuration.ffiConfig())
                    try handle.connect(params: configuration.peerParams())
                    self?.cryprq = handle
                    self?.pump = PacketPump(startingWith: self?.packetFlow, handle: handle)
                    completionHandler(nil)
                } catch {
                    completionHandler(error)
                }
            }
        } catch {
            completionHandler(error)
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        pump?.stop()
        cryprq?.close()
        cryprq = nil
        completionHandler()
    }
}
```

- `TunnelConfiguration` decodes options dictionary or uses `UserDefaults` to pull configuration from the companion app.
- `PacketPump` uses `packetFlow.readPackets` / `packetFlow.writePackets` on a `DispatchQueue` to bridge data to the FFI (currently no-op until data plane is ready).
- Handle errors gracefully and log via `os_log` or unified logging (`Logger`).

## Entitlements & Provisioning

- Packet Tunnel extension requires `com.apple.developer.networking.networkextension` with the `packet-tunnel-provider` entitlement.
- App host must include the same entitlement if it communicates with the provider.
- Provisioning:
  - Development: enable “Packet Tunnel” capability in Apple Developer portal.
  - Distribution: App Store provisioning profile with Network Extension entitlement.
- Add `NSUserTrackingUsageDescription` (if necessary) and privacy strings for App Store.

## Build & CI

- Use `xcodebuild` to produce per-platform static libs.
- Add GitHub Actions workflow (`apple-build.yml`) to:
  - Run `xcodebuild -scheme CrypRQPacketTunnel -destination 'generic/platform=iOS' archive`.
  - Run `xcodebuild -scheme CrypRQTunnelApp -destination 'generic/platform=macOS' build`.
  - Optionally produce `.xcframework` from `libcryprq_core.a`.
- Scripts:
  - `build-universal.sh`: build FFI static libs for `aarch64-apple-ios`, `x86_64-apple-ios`, `aarch64-apple-darwin`, `x86_64-apple-darwin`, and combine via `lipo` or `xcframework`.

## Testing

### Unit Tests

- `CrypRQTunnelKitTests`: mock `CrypRQFFI` to verify configuration encoding and lifecycle.
- `PacketTunnelProviderTests`: instantiate provider with fake `NEPacketTunnelFlow` to ensure start/stop flows.

### Manual Test Checklist (App Review oriented)

1. **Basic tunnel start/stop**: verify UI toggle starts the VPN, logs show connection.
2. **On-demand rules**: configure on-demand in settings, ensure NE reconnects after loss.
3. **Failsafe**: simulate `CrypRQHandle` error (invalid peer) and ensure provider reports error to host.
4. **Packet routing**: confirm DNS resolution goes through VPN (once data plane supports it).
5. **Background operation**: lock device, ensure tunnel stays alive for at least 10 minutes.
6. **Captive portal detection**: confirm provider handles network changes via `handleAppMessage`.

Document results in `docs/qa_apple.md` when ready.

## App Store Submission Notes

- Provide `VPN usage description` in `Info.plist` (e.g. `NSLocalNetworkUsageDescription`, `NSBonjourServices` if used).
- `App Privacy` section: declare “Data Not Collected”.
- For macOS notarization, integrate with existing DMG pipeline (see `docs/macos-notarization.md`).

---

With FFI and documentation in place, next steps are scaffolding the Xcode workspace, generating headers, and wiring the Swift bridge. Once basic start/stop flows work on both macOS and iOS simulators, we can layer on PacketPump functionality when the data-plane lands.

