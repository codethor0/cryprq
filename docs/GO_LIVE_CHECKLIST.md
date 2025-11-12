# Go-Live Cutover Checklist

**Zero fluff production cutover procedure for CrypRQ v0.1.0**

## 1. Freeze and Tag

```bash
git switch main
git pull --ff-only
git tag -a v0.1.0 -m "CrypRQ production cutover"
git push origin v0.1.0
```

## 2. Rebuild and Verify Locally

```bash
rustup toolchain install 1.83.0
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings
time cargo build --release -p cryprq
cargo test --all
```

## 3. Container Build & Smoke

```bash
docker build -t cryprq-node:release .
docker run --rm -p 9999:9999/udp cryprq-node:release \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1
## From a second shell:
docker run --rm --network host cryprq-node:release \
  --peer /ip4/127.0.0.1/udp/9999/quic-v1
```

**Expect**: QUIC handshake and liveness events.

## 4. Security Gates

```bash
./scripts/syft-sbom.sh
./scripts/grype-scan.sh
./scripts/repro-build.sh
```

These populate `release-*/security/` with SBOM, scan results, checksums, and reproducible build evidence.

## 5. Rotation/Limits Sanity

- **CRYPRQ_ROTATE_SECS=300** (default five-minute rotation)
- Backoff and inbound limits tuned via `CRYPRQ_MAX_INBOUND`, `CRYPRQ_BACKOFF_*`
- **Allowlist enforcement** (recommended for production): Run listener with one or more explicit peer IDs via `--allow-peer`

## 6. Paste-and-Run: Cutover Smoke

```bash
bash scripts/cutover_smoke.sh
```

This single script performs the minimal end-to-end production smoke:
- Build → container → handshake → rotation check → security scans → artifact bundle

## Operations Profile

### Targets (aligned to Phase G metrics)

- **Binary size**: ≤ 6.5 MB; alert at > 7.0 MB
- **Startup time**: ≤ 500 ms (hard); warn at > 450 ms
- **Reproducible build hash**: Exact match required
- **Security gates**: No Critical/High from Grype; no cargo audit advisories with deny severity; secret scan clean

### SLOs and Alerts (CI-enforced)

**Blocking on**:
- Build
- Tests
- Docker QA handshake
- SBOM + Grype
- Reproducible build

**Nightly perf job**: Fail on regression > +10% latency or +10% size

## Release Routine

```bash
./finish_qa_and_package.sh
```

This emits QA logs, checksums, SPDX SBOM, Grype report under `release-*/security/`.

Update `PRODUCTION_SUMMARY.md` with:
- Measured binary size
- Startup time
- Rotation interval proof (attach `rotation.txt` from the smoke run)

## Post-Go-Live Hardening (Next Sprint)

1. **Peer policy**: Enforce persistent allowlist and begin work on revocation/policy directory
2. **Metrics/health**: Expose minimal HTTP health/metrics endpoint gated to localhost or mTLS; document in README Roadmap
3. **Userspace WireGuard data-plane**: Complete forwarding path; add integration tests
4. **Mobile CI**: Keep Android/iOS builds non-blocking but artifact-producing (stubs if signing unavailable)
5. **PQ data-plane exploration**: Track candidates and test vectors; keep PQ handshake as the current, shipping control-plane

## Notes Mapped to the Repo

- **QUIC/libp2p handshake paths** and listener/dialer commands are the canonical run flow
- **Five-minute rotation** is the default; tune via `CRYPRQ_ROTATE_SECS`
- **SBOM and vulnerability reporting** are part of the release bundle process

## Quick Reference

```bash
## Full cutover smoke test
bash scripts/cutover_smoke.sh

## Individual checks
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p cryprq
cargo test --all
docker build -t cryprq-node:release .
bash scripts/syft-sbom.sh
bash scripts/grype-scan.sh
bash scripts/repro-build.sh
```

