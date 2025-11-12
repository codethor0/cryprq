# CrypRQ 1.1.0 Release PR

## What's in this release

- **Charts**: EMA smoothing + unit toggle (bytes/KB/MB) + auto-scaling + latency alert band (>250ms)
- **Diagnostics v1**: Redacted JSONL + timeline summary + metrics snapshot
- **Tray state dev hooks**: Testable tray state for CI assertions
- **Kill-switch**: Default ON (disconnect on app quit)
- **Rate-limited errors**: Max 1 error toast per 10s
- **Mobile Report Issue**: Redacted ZIP + confirmation toast
- **Security**: SBOM generation, npm audit, license checker

## Pre-merge gates

- [ ] `./scripts/generate-sbom.sh` completed
- [ ] `node store/validate.mjs` passed (if applicable)
- [ ] `cd gui && make test` (desktop) green
- [ ] Mobile Detox (Android + iOS) green
  - [ ] `cd mobile && npx detox test -c android.emu.debug --headless`
  - [ ] `cd mobile && npx detox test -c ios.sim.debug`
- [ ] `cd gui && npm audit --omit=dev --audit-level=high` → 0 high/critical
- [ ] `cd gui && npm run lint && npm run typecheck` green
- [ ] Chart bundle verification: `grep -R "Throughput (last 60s)" gui/dist` succeeds

## Post-merge checklist

- [ ] `./scripts/go-live.sh 1.1.0` executed
- [ ] `./scripts/verify-release.sh` green
- [ ] Confirm artifacts signed/notarized (macOS/Windows)
  - [ ] macOS: `spctl --assess --type open --verbose dist-package/*.dmg`
  - [ ] Windows: `signtool verify /pa dist-package/*.exe` (if signed)
- [ ] GitHub Release created with artifacts
- [ ] Post release notes in #announcements (Slack/email)
- [ ] Update `docs/FINAL_STATUS.md` with release date

## Testing

### Desktop
- [ ] Charts render and update at ~1 Hz
- [ ] EMA smoothing slider works (0-0.4)
- [ ] Unit toggle works (bytes/KB/MB)
- [ ] Latency alert shows when >250ms
- [ ] Kill-switch works (quit while connected → disconnect)
- [ ] Diagnostics export works and is redacted
- [ ] Report Issue flow works (copy path + open folder)

### Mobile
- [ ] Report Issue generates redacted ZIP
- [ ] Share sheet works and shows confirmation
- [ ] First-run consent flow works
- [ ] Privacy controls work

## Known issues

- [ ] None (or list any known issues)

## Rollback plan

If issues are detected:
1. **Desktop**: Unmark v1.1.0 "Latest" on GitHub Release; re-promote last stable
2. **Mobile**: Play Console rollout → pause; TestFlight → expire 1.1.0 build
3. If needed: Re-run `./scripts/go-live.sh <prev>` to cut back-out tag

## Day-0 monitoring

- [ ] On-call engineer assigned
- [ ] Day-0 on-call card reviewed (`docs/DAY0_ONCALL_CARD.md`)
- [ ] Observability checks scheduled (every 2h for first 24h)
- [ ] Support inbox monitoring active

---

**Release Manager**: [Name]  
**Date**: [Date]  
**Version**: 1.1.0

