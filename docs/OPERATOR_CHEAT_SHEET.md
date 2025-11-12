# Operator Cheat Sheet

**Copy-paste ready** - Run anytime, drop in PR comments

## Most-used commands

### 1) Validate + open PR (no ship)

```bash
./scripts/one-shot.sh
```

### 2) Validate + PR + ship after checks

```bash
SHIP=true ./scripts/one-shot.sh --ship
```

### 3) Add post-ship monitoring + sanity checks

```bash
SHIP=true RUN_POST=true ./scripts/one-shot.sh --ship --post
```

## Quick sanity before ship (30s)

```bash
./scripts/quick-smoke.sh || ./scripts/local-validate.sh
./scripts/observability-checks.sh
./scripts/sanity-checks.sh
```

## Golden path (60s)

**Desktop:** Connect → charts ≤3–5s → Rotate (toast ≤2s) → Disconnect

**Mobile:** Settings → Report Issue → share sheet (<2MB, "Report Prepared")

## Rollback (if needed)

**Desktop:** unmark v1.1.0 "Latest" on GitHub Release; re-promote prior stable

**Mobile:** pause Play rollout / expire TestFlight build

## Pro tip (flags & telemetry—on demand)

### Kill features at runtime (no rebuild)

```bash
CRYPRQ_FLAGS='{"enableCharts":false,"enableTrayEnhancements":false}' npm run dev
```

### Turn on telemetry v0 only when ready (in-app toggle)

**Then tail:**

```bash
tail -f ~/.cryprq/telemetry/events-$(date +%Y-%m-%d).jsonl
```

---

**You're set. Run the one-shot, watch CI publish, and go live. **

