# Security

CrypRQ targets adversaries capable of capturing encrypted traffic today and decrypting it later. It combines ML-KEM (Kyber768-compatible) with X25519 to strengthen the key exchange against store-now-decrypt-later scenarios.

## Threat Model
| Asset | Threat | Mitigation |
|-------|--------|------------|
| Session keys | Passive capture, future PQ decryption | Hybrid ML-KEM + X25519 exchange with 5-minute rotation |
| Control-plane | Man-in-the-middle, tampering | libp2p identity verification, QUIC transport |
| Supply-chain | Dependency compromise | Vendored `if-watch`, `cargo audit`, `cargo deny`, CodeQL |

### Trust Boundaries
- Each peer authenticates using libp2p identity keys; no implicit LAN trust.
- No centralized CA; admins distribute peer IDs securely.
- Network assumed hostile; encryption enforced on all control-plane traffic.

### Limitations
- Data-plane (userspace WireGuard) is experimental; packet forwarding is incomplete.
- No automated peer revocation list or ACLs.
- DoS protection limited to basic libp2p behaviour.
- Dependency `pqcrypto-mlkem` requires continuous review.

## Key Rotation
- Default rotation interval: 300 seconds (`CRYPRQ_ROTATE_SECS`).
- Old keys are overwritten to limit exposure.
- Rotation logs available at `info` level for monitoring.

## Responsible Disclosure
- Email `security@codethor0.com` (see [SECURITY.md](../SECURITY.md) for PGP).
- Acknowledge within 72 hours; plan shared within 7 days.

## Supply-Chain Hygiene
- CI runs `cargo fmt`, `cargo clippy`, `cargo test`, `cargo audit`, `cargo deny`, CodeQL.
- Reproducible build guide: [REPRODUCIBLE.md](../REPRODUCIBLE.md).
- Vendored dependencies stored in `third_party/`.

---

**Checklist**
- [ ] Reviewed current limitations before deployment.
- [ ] Configured rotation interval and monitoring.
- [ ] Established disclosure channel with `security@codethor0.com`.

