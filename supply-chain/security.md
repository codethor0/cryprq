# CrypRQ Supply Chain Security

## Threat Model

### Adversaries
1. **Global Passive Adversary** - NSA/GCHQ-level traffic analysis, mass surveillance
2. **Q-Day** - Quantum computer breaking current asymmetric crypto (RSA, ECDH)
3. **Physical Seizure** - Device confiscation, forensic memory extraction

### Mitigations
- **Post-Quantum Crypto**: Kyber768 key exchange (NIST finalist)
- **Perfect Forward Secrecy**: 5-minute key rotation with automatic zeroization
- **Zero-Trust Architecture**: No persistent keys, no PKI, no trusted third parties
- **Memory Safety**: Rust guarantees, explicit zeroize on key rotation
- **Userspace VPN**: No kernel modules, no root privileges required

## Verified Bootstrapping

### Prerequisites
- Rust 1.82.0 (pinned via rust-toolchain.toml)
- Git with GPG signature verification
- SHA256 checksum tools

### Reproducible Build
```bash
# 1. Verify Git commit signatures
git verify-commit HEAD
git log --show-signature -1

# 2. Clone with submodule verification
git clone --recurse-submodules https://github.com/codethor0/cryprq.git
cd cryprq

# 3. Verify rust-toolchain matches expected hash
sha256sum rust-toolchain.toml
# Expected: [HASH_TO_BE_FILLED]

# 4. Build with locked dependencies
cargo build --release --locked

# 5. Verify binary checksum
sha256sum target/release/cryprq
# Compare with release checksums.txt
```

### Nix Reproducible Build
```bash
nix-shell --pure --run "cargo build --release --locked"
sha256sum target/release/cryprq
```

### Docker Reproducible Build
```bash
docker build -t cryprq-build -f Dockerfile.reproducible .
docker run --rm cryprq-build sha256sum /build/target/release/cryprq
```

## Signed Release Process

### Prerequisites
- cosign (Sigstore keyless signing)
- GitHub CLI with repo permissions
- GPG key for commit signing

### Release Steps
1. **Tag & Sign Commit**
   ```bash
   git tag -s v0.x.y -m "Release v0.x.y"
   git push origin v0.x.y
   ```

2. **Build Static Binary**
   ```bash
   cargo zigbuild --release --target x86_64-unknown-linux-musl
   cargo zigbuild --release --target aarch64-unknown-linux-musl
   ```

3. **Generate SBOM**
   ```bash
   cargo cyclonedx --format json --output-pattern bom.json
   ```

4. **Sign with cosign (keyless)**
   ```bash
   cosign sign-blob --yes target/release/cryprq > cryprq.sig
   cosign sign-blob --yes bom.json > bom.json.sig
   ```

5. **Generate Checksums**
   ```bash
   sha256sum target/release/cryprq* > checksums.txt
   sha256sum bom.json >> checksums.txt
   ```

6. **Create GitHub Release**
   ```bash
   gh release create v0.x.y \
     target/release/cryprq-* \
     bom.json \
     bom.json.sig \
     checksums.txt \
     --title "v0.x.y" \
     --notes-file CHANGELOG.md
   ```

### Verification (End-User)
```bash
# 1. Download release
gh release download v0.x.y

# 2. Verify checksums
sha256sum -c checksums.txt

# 3. Verify cosign signature
cosign verify-blob --signature cryprq.sig \
  --certificate-identity=codethor0@users.noreply.github.com \
  --certificate-oidc-issuer=https://github.com/login/oauth \
  cryprq

# 4. Verify SBOM signature
cosign verify-blob --signature bom.json.sig \
  --certificate-identity=codethor0@users.noreply.github.com \
  --certificate-oidc-issuer=https://github.com/login/oauth \
  bom.json
```

## Dependency Verification

### Allowed Git Dependencies
- `rosenpass/rosenpass` (main branch) - PQ crypto primitives
- `rosenpass/memsec` (pinned commit) - Memory zeroization

### Cargo Audit Policy
- **DENY**: All vulnerabilities
- **DENY**: Unmaintained crates (>180 days no update)
- **DENY**: Yanked crates
- **WARN**: Informational advisories

### License Policy
- **ALLOW**: MIT, Apache-2.0, BSD-2/3-Clause, ISC, GPL-3.0
- **DENY**: AGPL-3.0, GPL-2.0, LGPL-*

## Incident Response

### Security Disclosure
Email: codethor@gmail.com (GPG: [KEY_ID])

### Response Timeline
- **Acknowledgment**: 24 hours
- **Triage**: 72 hours
- **Patch**: 7 days (critical), 30 days (high)
- **Public Disclosure**: 90 days or patch release

### CVE Process
1. Request CVE via GitHub Security Advisory
2. Coordinate disclosure with RustSec Advisory DB
3. Publish patch + security advisory simultaneously
4. Notify distros (Debian, Arch, Nix) 48h prior

## Audit History
- **[DATE]**: Initial security review (internal)
- **[DATE]**: Planned external audit (TBD)

## References
- [RustSec Advisory Database](https://rustsec.org/)
- [Sigstore Documentation](https://docs.sigstore.dev/)
- [NIST PQC Standards](https://csrc.nist.gov/projects/post-quantum-cryptography)
