# Store Listing Copy

Marketing copy tailored for Microsoft Store, Google Play, and Apple App Store. Each section includes short/long descriptions, screenshot captions, and keyword suggestions emphasizing CrypRQ’s post-quantum, open-source focus.

## Core Messaging Themes

- Hybrid ML-KEM (Kyber768-compatible) + X25519 handshake.
- Zero-trust, explicit peer allowlists and rapid key rotation.
- No trackers, no analytics, reproducible builds.
- Experimental data-plane (control-plane MVP).

## Microsoft Store

**Short description (80 characters max):**
> Post-quantum VPN control-plane with explicit peer trust.

**Long description:**
> CrypRQ delivers a post-quantum, zero-trust VPN control-plane for security-conscious teams. Pair Kyber768-compatible ML-KEM with X25519 to defend against store-now-decrypt-later threats, rotate keys every five minutes, and enforce explicit peer allowlists. CrypRQ is open-source, reproducibly built, and ships without telemetry or trackers. This release focuses on the control-plane; the data-plane remains experimental while we continue hardening the tunnel.

**Screenshot captions (5 × 1920×1080):**
1. “Hybrid ML-KEM + X25519 handshake in action.”
2. “Metrics and health checks with explicit allowlists.”
3. “Structured rotation logs for audit-ready environments.”
4. “CLI control with reproducible build metadata.”
5. “Security artifacts: SBOM + vulnerability scan outputs.”

**Keywords:**
`post-quantum vpn`, `zero trust`, `kyber`, `libp2p`, `wireguard`, `quic`, `security`, `open source`

## Google Play

**Short description (80 chars):**
> Post-quantum VPN control-plane with zero-trust peers.

**Full description:**
> CrypRQ pairs Kyber768-compatible ML-KEM with X25519 to create a post-quantum, zero-trust VPN control-plane. Explicit peer allowlists, rapid key rotation, and optional Prometheus metrics deliver insight without telemetry. CrypRQ is open-source, reproducibly built, and contains no trackers. This build focuses on the control-plane (VpnService tunnel); the data-plane remains experimental until GA.

**Screenshots (6 × 1080×1920):**
1. “Prominent disclosure: local tunnel, no third-party servers.”
2. “Peer allowlists and rotation cadence configuration.”
3. “Live metrics: rotations, handshakes, active peers.”
4. “Foreground notification keeps the tunnel visible.”
5. “Structured logs for debugging and audit trails.”
6. “Zero analytics: privacy-first by design.”

**Feature graphic text:**
> “CrypRQ – Post-Quantum Zero-Trust VPN”

**Tags / Keywords:**
`vpn`, `post-quantum`, `kyber`, `x25519`, `zero trust`, `telemetry-free`, `open source`, `vpnservice`

## Apple App Store

**Subtitle (30 chars):**
> Post-quantum VPN control-plane

**Promotional text (170 chars):**
> CrypRQ delivers a post-quantum, zero-trust VPN control-plane with explicit peer allowlists, rapid key rotation, and reproducible builds—no telemetry, no trackers.

**Description:**
> CrypRQ combines Kyber768-compatible ML-KEM with X25519 to secure control-plane traffic against future quantum adversaries. Configure explicit peer allowlists, observe rotations and handshakes via structured metrics, and rely on reproducible builds for supply-chain assurance. CrypRQ does not collect analytics or ship trackers. This release focuses on the control-plane; the data-plane remains experimental.

**Keywords:**
`postquantum`, `vpn`, `kyber`, `x25519`, `zerotrust`, `libp2p`, `quic`, `security`, `open source`

**Screenshot captions (6 × 1280×720 for iPad/macOS, 6 × 1242×2688 for iPhone):**
1. “Start Tunnel: Kyber768 + X25519 handshake.”
2. “Allowlist peers and rotation interval controls.”
3. “Health & metrics endpoint configuration.”
4. “Structured logs with rotation epoch tracking.”
5. “SBOM + vulnerability reporting workflow.”
6. “Reproducible build instructions inside the app.”

**App Privacy notes:**
- Data Not Collected.
- No third-party analytics.
- Optional metrics are local only.

## Localization

- Begin with `en-US`.
- Consider translating short/long descriptions for DE, FR, ES once release cadence stabilizes.

## Cross-Store Consistency

- Emphasize “control-plane only” status until data-plane is GA.
- Include link to GitHub repository and documentation site (where stores allow).
- Ensure privacy policy URL (`https://cryprq.dev/privacy.html`) is consistent.

## Next Steps

1. Drop copy into platform-specific metadata files (Play Console, App Store Connect, Microsoft Partner Center).
2. Align screenshots with UI once Android/iOS/macOS hosts are functional.
3. Version content alongside releases (`fastlane`/`store` directories).

