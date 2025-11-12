# Troubleshooting Guide

**Fast triage for common issues**

##  If Something Fails

### E2E Flaky?

**Re-run just Playwright with retries**:
```bash
cd gui && npm run test:playwright -- --retries=2 --reporter=list
```

**Check Playwright reports**:
```bash
# Open HTML report
open gui/playwright-report/index.html

# Or view in CI artifacts
```

### Fake Backend Didn't Come Up?

**Check logs**:
```bash
docker logs $(docker ps -q --filter name=fake-cryprq) --tail=200
```

**Check metrics endpoint**:
```bash
curl -s http://localhost:9464/metrics | head
```

**Restart fake backend**:
```bash
cd gui
docker compose -f docker-compose.yml down
docker compose -f docker-compose.yml up -d fake-cryprq
timeout 30 bash -c 'until curl -sf http://localhost:9464/metrics >/dev/null; do sleep 1; done'
```

### Bundle Mismatch in Verify Script?

**Check if chart text exists in built bundle**:
```bash
# After build
grep -r "Throughput (last 60s)" gui/dist/ || echo "Chart bundle missing"

# Rebuild if missing
cd gui
npm run build:mac    # macOS
npm run build:win    # Windows
make build-linux     # Linux
```

**Verify bundle includes recharts**:
```bash
grep -r "recharts" gui/dist/ || echo "Recharts not bundled"
```

### Tests Failing Locally But Pass in CI?

**Clear caches**:
```bash
cd gui
rm -rf node_modules .playwright
npm ci
npx playwright install --with-deps
```

**Check Node version**:
```bash
node --version  # Should be 18+
```

### Build Failing?

**Check platform-specific requirements**:
- **Linux**: `make` and `docker compose` required
- **macOS**: Xcode Command Line Tools required
- **Windows**: Visual Studio Build Tools required

**Check build logs**:
```bash
cd gui
npm run build:mac 2>&1 | tee build.log
# Review build.log for errors
```

### Docker Issues?

**Check Docker is running**:
```bash
docker ps
```

**Restart Docker services**:
```bash
# Stop all
cd gui && docker compose down
cd ../mobile && docker compose down || true

# Start fresh
cd gui && docker compose up -d fake-cryprq
```

**Check Docker logs**:
```bash
docker compose -f gui/docker-compose.yml logs fake-cryprq
```

### Git Push Failing?

**Check remote**:
```bash
git remote -v
```

**Check branch**:
```bash
git rev-parse --abbrev-ref HEAD
```

**Force push (if safe)**:
```bash
git push -u origin "$(git rev-parse --abbrev-ref HEAD)" --force-with-lease
```

### PR Creation Failing?

**Check GitHub CLI**:
```bash
gh auth status
```

**Create PR manually**:
```bash
# Get branch name
BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Open browser
open "https://github.com/[org]/cryprq/compare/main...$BRANCH"
```

##  Cleanup Commands

**Stop Docker services**:
```bash
cd gui && docker compose down
cd ../mobile && docker compose down || true
```

**Kill stray Electron dev processes**:
```bash
pkill -f electron || true
```

**Wipe transient artifacts (keeps reports)**:
```bash
rm -rf gui/node_modules mobile/node_modules gui/.playwright artifacts/desktop/* 2>/dev/null || true
```

**Full cleanup**:
```bash
./scripts/cleanup.sh
```

##  Quick Reference

**All checks**:
```bash
./scripts/observability-checks.sh && ./scripts/sanity-checks.sh
```

**Redaction check**:
```bash
grep -R -E "bearer |privKey=|authorization:" ~/.cryprq/logs || echo " OK"
```

**Session state timeline**:
```bash
jq -c 'fromjson | select(.event=="session.state") | [.ts,.data.state]' ~/.cryprq/logs/*.log | tail -20
```

**Recent errors**:
```bash
jq -c 'fromjson | select(.lvl=="error") | [.ts,.event,.msg]' ~/.cryprq/logs/*.log | tail -10
```

##  Common Error Patterns

### `PORT_IN_USE`
- **Cause**: UDP port already in use
- **Fix**: Change port in Settings → Transport → UDP Port

### `PROCESS_EXITED`
- **Cause**: CLI process crashed
- **Fix**: Check exit code in diagnostics, review logs

### `METRICS_UNREACHABLE`
- **Cause**: Fake backend not running or wrong endpoint
- **Fix**: Start fake backend, check endpoint URL

### `INVALID_ENDPOINT`
- **Cause**: REMOTE endpoint not in allowlist
- **Fix**: Add hostname to Settings → Security → Allowlist

---

**Still stuck?** Check `docs/DAY0_ONCALL_CARD.md` for escalation path.
