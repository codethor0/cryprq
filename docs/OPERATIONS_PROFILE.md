# Operations Profile

**What to keep green, continuously**

## Targets (Aligned to Phase G Metrics)

| Metric | Target | Alert Threshold | Hard Limit |
|--------|--------|-----------------|------------|
| Binary size | ≤ 6.5 MB | > 7.0 MB | N/A |
| Startup time | ≤ 500 ms | > 450 ms | > 500 ms (hard) |
| Reproducible build | Exact hash match | Hash mismatch | Hash mismatch |
| Security (Grype) | No Critical/High | Critical/High found | Critical/High found |
| Security (cargo audit) | No deny-severity advisories | Deny-severity found | Deny-severity found |
| Secret scan | Clean | Secrets found | Secrets found |

## SLOs and Alerts (CI-Enforced)

### Blocking Checks

These checks **must pass** for any release:

1. **Build**: `cargo build --release -p cryprq`
2. **Tests**: `cargo test --all`
3. **Docker QA handshake**: QUIC handshake verification in containers
4. **SBOM + Grype**: Security artifact generation and scanning
5. **Reproducible build**: Hash match validation

### Nightly Performance Job

**Fail on regression**:
- Latency increase > +10%
- Binary size increase > +10%

## Release Routine

### Automated Release Bundle

```bash
./finish_qa_and_package.sh
```

**Outputs**:
- `release-*/security/` - SBOM, Grype reports, checksums
- `release-*/qa/` - QA logs and test results
- `release-*/bin/` - Release binaries and checksums

### Manual Release Steps

1. **Run cutover smoke**:
   ```bash
   bash scripts/cutover_smoke.sh
   ```

2. **Update PRODUCTION_SUMMARY.md**:
   - Measured binary size
   - Startup time
   - Rotation interval proof (attach `rotation.txt` from smoke run)

3. **Tag release**:
   ```bash
   git tag -a v0.1.0 -m "CrypRQ production cutover"
   git push origin v0.1.0
   ```

## Monitoring

### Key Metrics to Track

- **Binary size**: Track over time to detect bloat
- **Startup time**: Monitor for regressions
- **Handshake latency**: Measure QUIC connection time
- **Rotation events**: Verify 5-minute cadence
- **Security scan results**: Track vulnerability trends

### Alert Conditions

- Binary size exceeds 7.0 MB
- Startup time exceeds 500 ms
- Security scan finds Critical/High vulnerabilities
- Reproducible build hash mismatch
- Test failures in CI

## Post-Go-Live Hardening

### Next Sprint Priorities

1. **Peer Policy**
   - Enforce persistent allowlist
   - Begin work on revocation/policy directory

2. **Metrics/Health**
   - Expose minimal HTTP health/metrics endpoint
   - Gate to localhost or mTLS
   - Document in README Roadmap

3. **Userspace WireGuard Data-Plane**
   - Complete forwarding path
   - Add integration tests

4. **Mobile CI**
   - Keep Android/iOS builds non-blocking
   - Artifact-producing (stubs if signing unavailable)

5. **PQ Data-Plane Exploration**
   - Track candidates and test vectors
   - Keep PQ handshake as current, shipping control-plane

## Operational Notes

- **QUIC/libp2p handshake paths** and listener/dialer commands are the canonical run flow
- **Five-minute rotation** is the default; tune via `CRYPRQ_ROTATE_SECS`
- **SBOM and vulnerability reporting** are part of the release bundle process
- **Allowlist enforcement** recommended for production (use `--allow-peer`)

## Quick Commands

```bash
## Full cutover smoke
bash scripts/cutover_smoke.sh

## Individual checks
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p cryprq
cargo test --all
docker build -t cryprq-node:release .
bash scripts/syft-sbom.sh
bash scripts/grype-scan.sh
bash scripts/repro-build.sh

## Release bundle
./finish_qa_and_package.sh
```

