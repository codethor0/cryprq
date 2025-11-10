# CrypRQ Privacy Policy

Effective date: 2025-11-10

CrypRQ is an open-source VPN/tunnel project focused on post-quantum, zero-trust connectivity. This policy explains how the application handles data across supported platforms (Android, iOS, macOS, Windows) and supporting services.

## Summary

- CrypRQ does **not** collect analytics, advertising identifiers, or personal data.
- Configuration remains local to your device. You control the peer allowlist and key rotation cadence.
- CrypRQ communicates only with peers you specify; there is no central service that records session metadata.
- Logs and metrics stay on-device unless you explicitly export them.

## Data We Process

| Category | Details | Storage |
|----------|---------|---------|
| Configuration | Peer multiaddrs, allowlisted peer IDs, rotation interval, MTU/routes (if configured). | Local device/app storage. |
| Runtime metrics | Rotation counters, handshake success/failure counts, active peer gauge. | Exposed via optional `/metrics` endpoint; not sent externally by default. |
| Logs | Structured application logs (info/debug). | Local log files or stdout; you control forwarding. |

We do not store session keys, payload traffic, or connection histories beyond in-memory state required for active connections.

## Peer Connectivity

- CrypRQ uses libp2p (QUIC/TCP) for control-plane traffic between peers you specify.
- Key exchange combines ML-KEM (Kyber768-compatible) and X25519, rotating every five minutes by default.
- Allowlisting enforces explicit trust; unauthorized peers are rejected locally.

## Telemetry & Advertising

- No analytics SDKs or advertising libraries are bundled.
- No unique identifiers are generated beyond libp2p peer IDs (derived from local keypairs).
- Push notifications are not used.

## Platform Disclosures

- **Google Play**: CrypRQ qualifies for the “No data collected” privacy label. Prominent disclosure informs users that a VPN tunnel is created locally and no traffic is routed to third-party servers.
- **Apple App Store**: `App Privacy` entry set to “Data Not Collected.” `NSPrivacyAccessedAPICategoryUserDefaults` declared only for local settings.
- **Microsoft Store**: Privacy policy link provided; no telemetry capabilities requested.
- **F-Droid**: Metadata declares no proprietary dependencies or tracking.

## Security

- Binaries are signed (Windows MSIX, macOS Developer ID) and reproducible build scripts are provided.
- SBOM and vulnerability reports accompany release artifacts.
- Responsible disclosure: security@codethor0.com.

## Third-Party Services

CrypRQ does not rely on third-party analytics or crash reporting. Optional integrations (e.g., Prometheus scrapers) are user-configured and outside the scope of this policy.

## Updates

We may update this policy to reflect new features (e.g., optional telemetry toggles). Changes will be versioned in the repository and linked from release notes.

## Contact

Thor Thor  
security@codethor0.com  
https://cryprq.dev

