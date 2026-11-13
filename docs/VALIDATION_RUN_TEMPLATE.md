# CrypRQ Validation Run Template

> **Historical document.** Written for the v1.0.x web preview, which ran an unauthenticated test-mode path. Statements about readiness or encryption in it do not describe the current code; see [README](../README.md) and [Security Notes](SECURITY_NOTES.md).

## STATUS: <GREEN/YELLOW/RED> (<short summary>)

**HEAD:**
- Commit: `<commit SHA>`
- Subject: `<commit subject>`
- Branch: `main`
- Timestamp (UTC): `<YYYY-MM-DD HH:MM:SS UTC>`

## WHAT I RAN (runtime):

- `git status -sb` → PASS/FAIL
- `git branch -a` → PASS/FAIL
- `cargo fmt --all --check` → PASS/FAIL
- `python3 scripts/governance_shield.py` → PASS/FAIL (note ENV LIMITATION if applies)
- `cargo test --workspace --lib` → PASS/FAIL/ENV LIMITATION
- `cd web && npm run build` → PASS/FAIL
- `docker compose -f docker/e2e/docker-compose.e2e.yml up --build --abort-on-container-exit` → PASS/FAIL/ENV LIMITATION
- `docker compose -f docker-compose.web.yml up --build -d` → PASS/FAIL/ENV LIMITATION
- `scripts/quick_smoke.sh` → PASS/FAIL/ENV LIMITATION
- `scripts/ci_e2e_probe.sh` → PASS/FAIL/ENV LIMITATION
- `scripts/release_probe.sh` → PASS/FAIL/ENV LIMITATION
- `scripts/vpn_host_probe.sh` → PASS/FAIL/ENV LIMITATION (if applicable)

## WHAT I OBSERVED (key evidence):

### QUIC/E2E:
- Test summary: Passed: X, Failed: Y, Skipped: Z
- Key log events:
  - `event=listener_ready ... status=accepting_connections`
  - `event=handshake_complete ... encryption=libp2p-transport cryprq_handshake=none status=ready`
  - `event=connection_established ... encryption_active=true`
  - `event=key_rotation status=success ... interval_secs=300`
  - Ping events: `result: Ok(...)`

### Web Stack:
- Container status: `<container name>` Up, port `<port>` mapped
- HTTP response: `<HTTP status code>` from `http://localhost:8787/`
- Web UI: Listener/Dialer modes tested, connection established, metrics updated

### VPN Host-Mode (if applicable):
- Environment: OS `<name>`, root privileges: YES/NO, TUN support: YES/NO
- Tunnel test: PASS/FAIL/ENV LIMITATION
- IP/DNS checks: PASS/FAIL/ENV LIMITATION (if enabled)

### Environment Limitations:
- List any ENV LIMITATION encountered (SSL/TLS issues, Docker daemon unavailable, npm EPERM, missing TUN/root, etc.)

## BUGS / ISSUES:

- None observed. No blocking bugs detected.
- OR: List each bug with ID (e.g., BUG-UI-1), area, severity, steps to reproduce, and evidence

## RE-VERIFY RESULT:

### QUIC/E2E:
- Status: GREEN/YELLOW/RED/ENV LIMITATION
- Summary: All 5 tests passing / X tests failed / E2E not run due to environment

### Web Stack:
- Status: GREEN/YELLOW/RED/ENV LIMITATION
- Summary: Container healthy, HTTP 200, UI functional / Issues observed / Not tested

### VPN Host-Mode:
- Status: GREEN/YELLOW/RED/ENV LIMITATION/NOT TESTED
- Summary: Tunnel established, routing correct / Issues observed / Not tested / Environment limitations

## HYGIENE:

- Working tree clean: YES/NO
- Branch model intact: YES/NO (only main locally, origin/main remotely)
- Emojis/prompt artifacts: Confirmed absent / Issues found
- Support bundle collected: YES/NO (if applicable)
- Session timeline exported: YES/NO (if applicable)

## NEXT ACTION:

- None: system validated and ready.
- OR: Specific follow-ups (e.g., investigate failing test X, address ENV limitation Y, fix bug Z)

## Additional Artifacts:

- Support bundle: `support_bundles/cryprq_support_YYYYMMDD_HHMMSS.tar.gz` (if collected)
- Session timeline: `cryprq_timeline_YYYYMMDD_HHMMSS.json` (if exported from Web Tester)
- Validation run report: `docs/VALIDATION_RUN_YYYYMMDD_HHMMSSZ_<SHA>.md` (if generated via `scripts/validation_run_report.sh`)

## Notes:

- This template follows the same structure as previous validation runs (e.g., `docs/VALIDATION_RUN_RELEASE_12f7bd6.md`)
- For automated report generation, use `scripts/validation_run_report.sh`
- For session timeline export, use the "Download Timeline (JSON)" button in the Web Tester Debug Console
- See `docs/RELEASE_CHECKLIST.md` for release-specific validation procedures

