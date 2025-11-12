# Quick Start: Desktop 1.1.0 Release

## 🚀 Fast Path (All-in-One)

```bash
# Run the go-live script
./scripts/go-live.sh 1.1.0
```

This script will:
1. ✅ Check GitHub secrets
2. ✅ Generate SBOM
3. ✅ Validate store listings
4. ✅ Run pre-release tests
5. ✅ Create release tag
6. ✅ Provide post-release instructions

---

## 📋 Manual Steps (If Needed)

### 1. Secrets Check
```bash
gh secret list
```

### 2. SBOM + Validation
```bash
./scripts/generate-sbom.sh
node store/validate.mjs
```

### 3. Pre-Release Tests
```bash
cd gui
make test
make build-linux
../scripts/smoke-tests.sh
```

### 4. Create Release
```bash
cd ..
./scripts/release.sh 1.1.0
git push origin v1.1.0
git push origin main
```

### 5. Post-Release Verification
```bash
# After CI completes
./scripts/verify-release.sh

# Run sanity checks
./scripts/sanity-checks.sh
```

---

## 📱 Mobile Release

See `docs/MOBILE_RELEASE.md` for complete mobile release path.

---

## 🆘 Need Help?

- **Go-Live Guide:** `docs/GO_LIVE_SEQUENCE.md`
- **Incident Runbook:** `docs/INCIDENT_RUNBOOK.md`
- **Mobile Release:** `docs/MOBILE_RELEASE.md`

---

**Last Updated:** 2025-01-15

