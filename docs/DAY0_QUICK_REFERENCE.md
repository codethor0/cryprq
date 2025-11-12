# Day-0 Quick Reference

**One-page cheat sheet for release day**

## 🚀 Ship Command

```bash
./scripts/go-live.sh 1.1.0 && ./scripts/verify-release.sh
```

## ✅ Golden Path

1. Connect → Status "Connected" (≤2s)
2. Rotate → Toast appears (≤2s)
3. Disconnect → Status "Disconnected" (immediate)

## 🔍 Quick Checks (Every 2h)

```bash
## 1. Observability
./scripts/observability-checks.sh  # All PASS

## 2. Redaction
grep -r "bearer \|privKey=\|authorization:" ~/.cryprq/logs  # No hits

## 3. Support inbox
## Check: 0 unresolved "can't connect" reports
```

## 🚨 Rollback

**Desktop**: GitHub Release → Unmark "Latest" → Re-promote previous  
**Mobile**: Play Console → Pause rollout | TestFlight → Expire build

## 📋 Files

- **On-Call Card**: `docs/DAY0_ONCALL_CARD.md`
- **PR Template**: `.github/PULL_REQUEST_TEMPLATE/release.md`
- **Announcement**: `docs/RELEASE_ANNOUNCEMENT.md`

## 📞 Escalation

- **Level 1**: On-Call Engineer (observability checks)
- **Level 2**: Engineering Lead (>1% crash rate, >5 tickets/2h)
- **Level 3**: CTO/Founder (data breach, service outage)

---

**Full Details**: See `docs/DAY0_ONCALL_CARD.md`

