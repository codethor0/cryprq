# Roadmap

> **📋 Master Prompt Roadmap:** For comprehensive post-quantum feature implementation plan aligned with the Master Prompt requirements, see [`MASTER_PROMPT_ROADMAP.md`](./MASTER_PROMPT_ROADMAP.md) and [`MASTER_PROMPT_QUICK_REF.md`](./MASTER_PROMPT_QUICK_REF.md).

## Near-Term
- Complete userspace WireGuard forwarding path.
- Implement peer directory and ACL enforcement.
- Add metrics endpoint and health probe.
- Improve QUIC/TCP dual-stack fallback handling.
- Publish crates with versioned releases.

## Medium-Term
- Explore PQ data-plane ciphers (e.g., PQ-TLS, PQ-WireGuard variants).
- Provide operator dashboard for session status and alerts.
- Add configurable rate limiting and DoS protections.
- Expand reproducible build coverage (additional architectures).

## Long-Term
- Federated CrypRQ deployments with policy-driven routing.
- Formal verification of key rotation and handshake logic.
- Managed updater with signed artifact verification.

---

**Checklist**
- [ ] Reviewed near-term goals.
- [ ] Aligned deployment plans with roadmap.
- [ ] Revisit roadmap quarterly.

