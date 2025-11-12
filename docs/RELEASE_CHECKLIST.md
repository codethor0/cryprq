# Release Checklist: Desktop 1.1.0

## ✅ Pre-Release

### Secrets Configuration
- [ ] `APPLE_ID` configured
- [ ] `APPLE_APP_SPECIFIC_PASSWORD` configured
- [ ] `APPLE_TEAM_ID` configured
- [ ] `CSC_LINK` configured (Windows)
- [ ] `CSC_KEY_PASSWORD` configured (Windows)

**Check:** `gh secret list`

### SBOM & Validation
- [ ] SBOM generated: `./scripts/generate-sbom.sh`
- [ ] Store listings validated: `node store/validate.mjs`
- [ ] Privacy policy URL live and accessible

### Pre-Release Tests
- [ ] Dockerized tests pass: `cd gui && make test`
- [ ] Local Linux build succeeds: `cd gui && make build-linux`
- [ ] Smoke tests pass: `./scripts/smoke-tests.sh`

### Code Quality
- [ ] All tests pass
- [ ] No linter errors
- [ ] TypeScript compiles without errors
- [ ] CHANGELOG.md updated for version

---

## 🏷️ Release

### Tag Creation
- [ ] Version bumped in `gui/package.json`
- [ ] CHANGELOG.md has entry for version
- [ ] Release tag created: `./scripts/release.sh 1.1.0`
- [ ] Tag pushed: `git push origin v1.1.0`
- [ ] Commits pushed: `git push origin main`

### CI Monitoring
- [ ] GitHub Actions workflow triggered
- [ ] Linux build completes successfully
- [ ] Windows build completes successfully
- [ ] macOS build completes successfully
- [ ] Artifacts uploaded to GitHub Release

---

## 🔒 Post-Release Verification

### Artifact Verification
- [ ] macOS DMG passes Gatekeeper: `spctl --assess --type open --verbose dist-package/*.dmg`
- [ ] Windows EXE is signed: `signtool verify /pa dist-package/*.exe`
- [ ] Linux artifacts have checksums

**Run:** `./scripts/verify-release.sh`

### Observability
- [ ] Observability checks pass: `./scripts/observability-checks.sh`
- [ ] Diagnostics export works
- [ ] No secrets in diagnostics ZIP (grep verification)
- [ ] Structured logs (JSONL v1) present

### Sanity Checks
- [ ] Kill-switch works: quit while connected → session stops ≤1s
- [ ] HTTPS enforcement: REMOTE http:// blocked
- [ ] Redaction verified: no bearer/privKey/authorization in diagnostics
- [ ] Crash symbols uploaded (dSYM + ProGuard mapping)

**Run:** `./scripts/sanity-checks.sh`

---

## 📱 Mobile Release (Separate)

### Secrets
- [ ] `ANDROID_KEYSTORE_*` configured
- [ ] `PLAY_JSON_KEY` configured
- [ ] `APP_STORE_CONNECT_API_KEY` configured

### Tests
- [ ] Android E2E tests pass
- [ ] iOS E2E tests pass (macOS)

### Release
- [ ] Mobile tag created: `git tag mobile-v1.0.0`
- [ ] Tag pushed: `git push origin mobile-v1.0.0`
- [ ] Artifacts uploaded (AAB + IPA)

### Staged Rollout
- [ ] Android: Internal → Closed → Production (10% → 50% → 100%)
- [ ] iOS: TestFlight (100 users) → App Review

---

## 🚨 Post-Release Monitoring

### Week 1
- [ ] Daily monitoring of crash reports
- [ ] Review user feedback
- [ ] Address critical issues
- [ ] Monitor health gates (if mobile)

### Week 2-4
- [ ] Weekly monitoring
- [ ] Performance optimization
- [ ] Feature requests review
- [ ] Plan next release

---

## 📋 Quick Reference

### All-in-One Go-Live
```bash
./scripts/go-live.sh 1.1.0
```

### Manual Steps
```bash
## 1. Secrets check
gh secret list

## 2. SBOM + validation
./scripts/generate-sbom.sh
node store/validate.mjs

## 3. Pre-release tests
cd gui && make test && make build-linux && ../scripts/smoke-tests.sh

## 4. Release
cd .. && ./scripts/release.sh 1.1.0
git push origin v1.1.0
git push origin main

## 5. Post-release verification
./scripts/verify-release.sh
./scripts/sanity-checks.sh
```

### Documentation
- **Go-Live Guide:** `docs/GO_LIVE_SEQUENCE.md`
- **Incident Runbook:** `docs/INCIDENT_RUNBOOK.md`
- **Mobile Release:** `docs/MOBILE_RELEASE.md`
- **Quick Start:** `docs/QUICK_START.md`

---

**Last Updated:** 2025-01-15

