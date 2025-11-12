# Master Prompt Implementation Roadmap

**Last Updated:** 2025-11-11  
**Status:** Assessment Complete → Implementation Planning

## Executive Summary

This document maps the Master Prompt requirements against CrypRQ's current implementation status and provides a prioritized roadmap for achieving full compliance with the post-quantum, zero-trust VPN vision.

---

## Current Implementation Status

###  Completed Features

#### Post-Quantum Cryptography (PQC)
-  **Hybrid ML-KEM (Kyber768) + X25519 handshake**
  - Location: `crypto/src/hybrid.rs`, `p2p/src/lib.rs`
  - Status: Implemented and tested
  - Library: `pqcrypto-mlkem` (ML-KEM 768)

-  **Five-minute key rotation**
  - Location: `node/src/lib.rs` (line 565), `node/src/rotate.rs`
  - Status: Implemented with secure zeroization
  - Mechanism: Tokio interval timer (300 seconds)

-  **Secure key zeroization**
  - Location: `node/src/lib.rs` (uses `zeroize` crate)
  - Status: Implemented for ephemeral keys

#### Platform-Specific Development
-  **Android**: `VpnService` module (`android/`)
  - Status: Scaffolded with JNI bridge
  - FFI: Rust core exposed via C ABI

-  **iOS/macOS**: Network Extension (`apple/`)
  - Status: SwiftPM package scaffolded
  - Architecture: `NEPacketTunnelProvider`

-  **Windows**: MSIX packaging (`windows/`)
  - Status: Packaging scripts and manifests prepared

-  **Linux**: Reproducible builds (musl, Nix)
  - Status: Docker and Nix flakes configured

#### Security & Supply Chain
-  **Supply-chain hardening**
  - `cargo audit`, `cargo deny`, CodeQL workflows
  - SPDX SBOM generation (Syft)
  - Vulnerability scanning (Grype)

-  **Reproducible builds**
  - Linux (musl), macOS, Nix, Docker
  - Documentation: `REPRODUCIBLE.md`

#### Cross-Platform GUI
-  **Desktop GUI** (Electron/React)
  - Status: v1.0.1 released
  - Features: Dashboard, Peers, Settings, Logs, Diagnostics

-  **Mobile GUI** (React Native)
  - Status: Bootstrap complete (M1-M15)
  - Architecture: Controller mode with profiles

---

## Gap Analysis

###  Critical Gaps (High Priority)

#### 1. Post-Quantum Pre-Shared Keys (PPKs)
**Status:**  Not Implemented  
**Priority:** High  
**Impact:** Enhanced security for peer authentication

**Requirements:**
- Implement PPK derivation using ML-KEM shared secrets
- Store PPKs securely (encrypted at rest)
- Rotate PPKs independently of session keys
- Support PPK-based peer authentication

**Implementation Plan:**
```rust
// New module: crypto/src/ppk.rs
pub struct PostQuantumPSK {
    key: [u8; 32],
    derived_from: KyberPublicKey,
    expires_at: SystemTime,
}

pub fn derive_ppk(
    kyber_shared: &[u8; 32],
    peer_id: &PeerId,
    salt: &[u8; 16],
) -> PostQuantumPSK {
    // BLAKE3 KDF with peer identity
    // Expires with key rotation
}
```

**Files to Create:**
- `crypto/src/ppk.rs`
- `crypto/src/lib.rs` (export PPK types)
- `p2p/src/lib.rs` (integrate PPK auth)
- `docs/ppk.md` (user guide)

**Estimated Effort:** 2-3 weeks

---

#### 2. User Toggle for Post-Quantum Encryption
**Status:**  Not Implemented  
**Priority:** High  
**Impact:** User control and compliance

**Requirements:**
- Settings UI toggle: "Enable Post-Quantum Encryption"
- Default: ON (post-quantum enabled)
- Runtime toggle (no restart required)
- Clear tooltips explaining benefits/limitations

**Implementation Plan:**

**Desktop GUI (`gui/src/components/Settings/Settings.tsx`):**
```typescript
<Section title="Security">
  <Toggle
    label="Post-Quantum Encryption"
    checked={settings.postQuantumEnabled}
    onChange={(enabled) => updateSettings({ postQuantumEnabled: enabled })}
    tooltip="Enable ML-KEM (Kyber768) + X25519 hybrid handshake for future-proof security. Disabling falls back to X25519-only (not recommended)."
  />
  <InfoBox>
    Post-quantum encryption protects against future quantum computer attacks.
    Recommended: Keep enabled for maximum security.
  </InfoBox>
</Section>
```

**Mobile (`mobile/src/screens/SettingsScreen.tsx`):**
- Similar toggle with platform-native UI
- Stored in MMKV (encrypted)

**Backend (`cli/src/main.rs`):**
```rust
#[command]
struct Config {
    #[arg(long, default_value = "true")]
    post_quantum: bool,
}

// In handshake logic:
if config.post_quantum {
    // Use hybrid ML-KEM + X25519
} else {
    // Fallback to X25519-only (with warning)
}
```

**Files to Modify:**
- `gui/src/components/Settings/Settings.tsx`
- `gui/src/types/index.ts` (add `postQuantumEnabled`)
- `gui/electron/main/settings.ts` (persist setting)
- `mobile/src/screens/SettingsScreen.tsx`
- `cli/src/main.rs` (CLI flag)
- `p2p/src/lib.rs` (conditional PQC)

**Files to Create:**
- `docs/post-quantum-toggle.md` (user guide)
- `docs/security-fallback.md` (technical details)

**Estimated Effort:** 1 week

---

#### 3. Additional PQC Algorithms (OQS Integration)
**Status:**  Not Implemented  
**Priority:** Medium  
**Impact:** Algorithm diversity and future-proofing

**Requirements:**
- Integrate Open Quantum Safe (OQS) library
- Support multiple PQC algorithms:
  - ML-KEM (Kyber) -  Already implemented
  - ML-DSA (Dilithium) - For signatures
  - SPHINCS+ - For hash-based signatures
- Algorithm selection via configuration
- Fallback chain: ML-KEM → X25519 → Error

**Implementation Plan:**

**Dependencies (`Cargo.toml`):**
```toml
[dependencies]
oqs = { git = "https://github.com/open-quantum-safe/liboqs-rust" }
## Or use individual crates:
dilithium = "0.1"
sphincsplus = "0.1"
```

**New Module (`crypto/src/pqc_suite.rs`):**
```rust
pub enum PQCAlgorithm {
    MLKEM768,      // Current default
    MLKEM1024,     // Higher security
    Dilithium3,    // For signatures
    SPHINCSPlus,   // Hash-based backup
}

pub struct PQCSuite {
    kex: PQCAlgorithm,
    sig: PQCAlgorithm,
}

impl PQCSuite {
    pub fn default() -> Self {
        Self {
            kex: PQCAlgorithm::MLKEM768,
            sig: PQCAlgorithm::Dilithium3,
        }
    }
}
```

**Files to Create:**
- `crypto/src/pqc_suite.rs`
- `crypto/src/dilithium.rs` (signature support)
- `docs/pqc-algorithms.md` (algorithm guide)

**Files to Modify:**
- `crypto/src/hybrid.rs` (support multiple algorithms)
- `Cargo.toml` (add OQS dependencies)
- `docs/security.md` (update algorithm list)

**Estimated Effort:** 3-4 weeks

---

###  Important Gaps (Medium Priority)

#### 4. Fuzz Testing Infrastructure
**Status:**  Not Implemented  
**Priority:** Medium  
**Impact:** Security robustness

**Requirements:**
- Fuzz tests for cryptographic operations
- Fuzz tests for protocol parsing
- CI integration (cargo-fuzz)
- Coverage reporting

**Implementation Plan:**

**Setup (`fuzz/Cargo.toml`):**
```toml
[package]
name = "cryprq-fuzz"
version = "0.0.0"

[dependencies]
libfuzzer-sys = "0.4"
cryprq-crypto = { path = "../crypto" }
cryprq-p2p = { path = "../p2p" }
```

**Fuzz Targets:**
- `fuzz/fuzz_targets/hybrid_handshake.rs`
- `fuzz/fuzz_targets/protocol_parse.rs`
- `fuzz/fuzz_targets/key_rotation.rs`

**CI Integration (`.github/workflows/fuzz.yml`):**
```yaml
name: Fuzz Testing
on: [push, pull_request]
jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rust-fuzz/cargo-fuzz-action@v1
        with:
          command: build
          args: --all-targets
```

**Files to Create:**
- `fuzz/Cargo.toml`
- `fuzz/fuzz_targets/` (multiple targets)
- `.github/workflows/fuzz.yml`
- `docs/fuzzing.md`

**Estimated Effort:** 2 weeks

---

#### 5. User Documentation & Tooltips
**Status:**  Partial  
**Priority:** Medium  
**Impact:** User education and adoption

**Current State:**
-  Technical docs exist (`docs/`)
-  User-facing tooltips missing
-  In-app help system missing
-  Tutorial/onboarding missing

**Requirements:**
- In-app tooltips for PQC features
- "What is Post-Quantum?" help screen
- First-run tutorial
- Contextual help in Settings

**Implementation Plan:**

**Desktop GUI (`gui/src/components/Help/PostQuantumInfo.tsx`):**
```typescript
export const PostQuantumInfo: React.FC = () => (
  <Modal>
    <h2>Post-Quantum Encryption</h2>
    <p>
      CrypRQ uses ML-KEM (Kyber768) + X25519 hybrid encryption to protect
      against future quantum computer attacks.
    </p>
    <ul>
      <li> Future-proof security</li>
      <li> Defense-in-depth (hybrid approach)</li>
      <li> Automatic key rotation every 5 minutes</li>
    </ul>
    <Link to="/docs/post-quantum">Learn more →</Link>
  </Modal>
);
```

**Mobile (`mobile/src/components/InfoTooltip.tsx`):**
- Native tooltip component
- Accessible (screen reader support)

**Files to Create:**
- `gui/src/components/Help/PostQuantumInfo.tsx`
- `gui/src/components/Help/FirstRunTutorial.tsx`
- `mobile/src/components/InfoTooltip.tsx`
- `docs/user-guide/post-quantum.md`
- `docs/user-guide/getting-started.md`

**Files to Modify:**
- `gui/src/components/Settings/Settings.tsx` (add help icons)
- `mobile/src/screens/SettingsScreen.tsx` (add tooltips)

**Estimated Effort:** 1-2 weeks

---

#### 6. Bug Bounty Program Setup
**Status:**  Not Implemented  
**Priority:** Low (can be deferred)  
**Impact:** Community security testing

**Requirements:**
- Bug bounty policy document
- Responsible disclosure process
- Integration with HackerOne/Bugcrowd (optional)
- Security contact information

**Implementation Plan:**

**Files to Create:**
- `SECURITY.md` (update with bounty info)
- `docs/bug-bounty.md`
- `.github/SECURITY.md` (GitHub security policy)

**Content:**
- Scope (what's in/out of scope)
- Rewards structure
- Reporting process
- Response SLA

**Estimated Effort:** 1 week (documentation)

---

###  Nice-to-Have Enhancements

#### 7. Automated Penetration Testing
**Status:**  Not Implemented  
**Priority:** Low  
**Impact:** Continuous security validation

**Options:**
- Third-party services (Synack, Cobalt)
- Internal red team exercises
- Automated security scanners (OWASP ZAP, Burp)

**Estimated Effort:** Ongoing (external service)

---

#### 8. Community Contribution Guidelines
**Status:**  Partial (`CONTRIBUTING.md` exists)  
**Priority:** Low  
**Impact:** Open source engagement

**Enhancements:**
- PQC-specific contribution guidelines
- Code review checklist for crypto code
- Security review process

**Estimated Effort:** 1 week (documentation)

---

## Implementation Roadmap

### Phase 1: Critical Features (Weeks 1-6)

**Week 1-2: User Toggle for PQC**
- Implement settings toggle (desktop + mobile)
- Add CLI flag
- Update documentation
- **Deliverable:** Users can enable/disable PQC encryption

**Week 3-5: Post-Quantum Pre-Shared Keys**
- Design PPK derivation scheme
- Implement PPK storage (encrypted)
- Integrate PPK authentication
- **Deliverable:** PPK-based peer authentication

**Week 6: Testing & Validation**
- Unit tests for PPK
- Integration tests for toggle
- Update security documentation

---

### Phase 2: Algorithm Diversity (Weeks 7-10)

**Week 7-9: OQS Integration**
- Integrate Dilithium for signatures
- Add SPHINCS+ support
- Algorithm selection UI
- **Deliverable:** Multiple PQC algorithms available

**Week 10: Testing & Documentation**
- Algorithm comparison tests
- Performance benchmarks
- User guide updates

---

### Phase 3: Security Hardening (Weeks 11-14)

**Week 11-12: Fuzz Testing**
- Set up cargo-fuzz infrastructure
- Create fuzz targets
- CI integration
- **Deliverable:** Automated fuzz testing in CI

**Week 13-14: Documentation & UX**
- In-app tooltips
- First-run tutorial
- Help system
- **Deliverable:** User-friendly PQC education

---

### Phase 4: Community & Transparency (Weeks 15+)

**Week 15: Bug Bounty Setup**
- Policy documentation
- Security contact
- Disclosure process

**Ongoing:**
- Community engagement
- Security audits
- Algorithm updates

---

## Success Metrics

### Technical Metrics
-  PPK implementation with <100ms overhead
-  PQC toggle works without restart
-  3+ PQC algorithms supported
-  Fuzz tests cover 80%+ of crypto code
-  Zero critical vulnerabilities in audits

### User Metrics
-  90%+ users keep PQC enabled (default)
-  Tooltip engagement >50%
-  Support tickets for PQC <5% of total

### Security Metrics
-  Bug bounty submissions: 5+ per quarter
-  Security audit findings: <3 critical per year
-  Fuzz test coverage: 80%+ of attack surface

---

## Risk Assessment

### High Risk
- **PPK Implementation Complexity**
  - Mitigation: Phased rollout, extensive testing
  - Fallback: Standard key exchange if PPK fails

- **OQS Library Maturity**
  - Mitigation: Use stable OQS releases, maintain forks
  - Fallback: Current ML-KEM implementation

### Medium Risk
- **Performance Impact of Multiple Algorithms**
  - Mitigation: Benchmarking, algorithm selection UI
  - Fallback: Default to ML-KEM only

- **User Confusion with Toggle**
  - Mitigation: Clear tooltips, default ON, warnings
  - Fallback: Remove toggle if adoption <50%

---

## Dependencies & Blockers

### External Dependencies
- OQS Rust bindings (if using OQS library)
- Fuzz testing infrastructure (cargo-fuzz)
- Bug bounty platform (if using HackerOne/Bugcrowd)

### Internal Blockers
- None identified (all features can be implemented incrementally)

---

## Next Steps

1. **Immediate (This Week):**
   - Review and approve this roadmap
   - Prioritize Phase 1 features
   - Assign implementation tasks

2. **Short-term (Next 2 Weeks):**
   - Start user toggle implementation
   - Design PPK derivation scheme
   - Set up fuzz testing infrastructure

3. **Long-term (Next Quarter):**
   - Complete Phase 1-2 features
   - Begin security audits
   - Launch bug bounty program

---

## References

- [Master Prompt Requirements](#) (this document's source)
- [Current Implementation](../README.md)
- [Security Model](../docs/security.md)
- [Roadmap](../docs/roadmap.md)
- [Open Quantum Safe Project](https://openquantumsafe.org/)

---

**Document Owner:** Development Team  
**Review Frequency:** Monthly  
**Last Review:** 2025-11-11

