# ## CrypRQ Apple Host Scaffold

This directory contains the initial Swift package and scripts required to integrate `cryp-rq-core` with Apple platforms (iOS/macOS).

### Layout

- `Package.swift` – SwiftPM definition for `CrypRQTunnelKit`, a reusable layer addressing FFI and tunnel orchestration.
- `Sources/CrypRQTunnelKit/` – Swift code for:
  - `CrypRQFFI` – safe wrapper around the C ABI (dynamic symbol lookup for now).
  - `CrypRQTunnelController` – observable state machine for Packet Tunnel providers.
  - `TunnelConfiguration` – converts host settings into FFI configs + `NEPacketTunnelNetworkSettings`.
  - `PacketPump` – placeholder abstraction for future packet bridging.
- `Tests/` – Unit tests validating controller state transitions with stub handles.
- `Scripts/`
  - `generate-headers.sh` – runs `cbindgen` to emit `cryprq_core.h` in `apple/Shared/include`.
  - `build-universal.sh` – placeholder for future xcframework/universal builds.

### Prerequisites

- Xcode 15.4 or newer
- SwiftPM (bundled with Xcode)
- Rust toolchain 1.83.0 (for header generation via `cbindgen`)

### Usage

1. Generate headers for the Swift bridge:
   ```bash
   ./apple/Scripts/generate-headers.sh
   ```
2. Add `cryprq_core.h` and the produced static libraries to your Xcode workspace (host app + Packet Tunnel extension).
3. Add `CrypRQTunnelKit` as a Swift Package dependency in the workspace.
4. Wire the Packet Tunnel Provider to `CrypRQTunnelController` and `PacketPump` (see `docs/apple.md` for detailed steps).

### Next Steps

- Fill in `PacketPump` with real queue bridging once the data-plane APIs are ready.
- Create the Xcode workspace (host app + extension) and reference the Swift package.
- Implement end-to-end integration tests using `NEPacketTunnelProvider` in a test host app.

