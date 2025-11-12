# Master Prompt - Quick Reference

**Last Updated:** 2025-11-11

## Current Status:  6/12 Features Complete

###  Implemented
1. Hybrid ML-KEM (Kyber768) + X25519 handshake
2. Five-minute key rotation with zeroization
3. Platform implementations (Android, iOS, macOS, Windows, Linux)
4. Supply-chain security (audit, deny, CodeQL, SBOM)
5. Reproducible builds
6. Cross-platform GUIs (Desktop v1.0.1, Mobile bootstrap)

###  Missing (Critical)
1. **Post-Quantum Pre-Shared Keys (PPKs)** - 2-3 weeks
2. **User Toggle for PQC** - 1 week
3. **Additional OQS Algorithms** - 3-4 weeks

###  Partial
4. **Fuzz Testing** - 2 weeks
5. **User Documentation/Tooltips** - 1-2 weeks
6. **Bug Bounty Program** - 1 week (docs)

---

## Quick Implementation Guide

### 1. User Toggle (Week 1)
```typescript
// gui/src/components/Settings/Settings.tsx
<Toggle
  label="Post-Quantum Encryption"
  checked={settings.postQuantumEnabled}
  onChange={updatePQC}
  tooltip="Enable ML-KEM + X25519 hybrid"
/>
```

### 2. PPKs (Weeks 3-5)
```rust
// crypto/src/ppk.rs
pub fn derive_ppk(kyber_shared: &[u8; 32], peer_id: &PeerId) -> PostQuantumPSK
```

### 3. OQS Integration (Weeks 7-9)
```toml
## Cargo.toml
dilithium = "0.1"
sphincsplus = "0.1"
```

---

## Priority Order

1. **User Toggle** (High impact, low effort)
2. **PPKs** (High security value)
3. **Fuzz Testing** (Security robustness)
4. **OQS Algorithms** (Future-proofing)
5. **Documentation** (User education)
6. **Bug Bounty** (Community engagement)

---

## Full Roadmap

See [`docs/MASTER_PROMPT_ROADMAP.md`](./MASTER_PROMPT_ROADMAP.md) for complete details.

---

**Quick Links:**
- [Full Roadmap](./MASTER_PROMPT_ROADMAP.md)
- [Security Model](./security.md)
- [Current Features](../README.md#features)

