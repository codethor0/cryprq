# Local Validation Guide

## Quick Start

Run the one-shot local validation script:

```bash
./scripts/local-validate.sh
```

This will:
1.  Spin up fake backend in Docker
2.  Run desktop GUI tests (lint, typecheck, unit, Playwright E2E)
3.  Build desktop artifacts (Linux/macOS/Windows)
4.  Run smoke tests
5.  Optionally run mobile Android CI smoke locally
6.  Commit, push feature branch, and create PR
7.  Print summary with artifact paths and logs

## What It Does

### 1. Repo Hygiene
- Checks current branch
- Creates feature branch if on main/master: `chore/local-validate-YYYYMMDD-HHMM`
- Verifies required directories exist

### 2. Desktop Validation
- **Fake Backend**: Starts Docker container, waits for `http://localhost:9464/metrics`
- **Dependencies**: Runs `npm install`
- **Static Checks**: `npm run lint` + `npm run typecheck`
- **Tests**: `npm run test:unit` + `npm run test:playwright`
- **Docker CI**: `make test` (optional)
- **Build**: Platform-specific build (`make build-linux` / `npm run build:mac` / `npm run build:win`)
- **Smoke Tests**: Runs `smoke-tests.sh`, `observability-checks.sh`, `sanity-checks.sh`
- **Artifacts**: Copies to `artifacts/desktop/<platform>/`
- **Reports**: Copies test reports to `artifacts/reports/`

### 3. Manual GUI Sanity
- Prompts for manual verification:
  - Connect → verify charts render (3-5s)
  - Toggle EMA smoothing → verify curve changes
  - Report Issue → verify diagnostics export

### 4. Mobile (Optional)
- Installs dependencies
- Starts mobile fake backend
- Builds Android debug APK
- Runs Detox E2E (if emulator available)
- Copies artifacts to `artifacts/mobile/`

### 5. Git Push
- Stages changes (excluding artifacts)
- Commits: "chore: local Docker validation, desktop build + test logs"
- Pushes branch to origin
- Creates PR via GitHub CLI (if available)

### 6. Summary
- Prints comprehensive summary with:
  - Docker container status
  - Test results
  - Artifact paths
  - Report locations
  - Branch and PR info
  - Next steps

## Prerequisites

- Docker and docker compose installed and running
- Node 18+ and npm available
- Git configured with push access
- (Optional) GitHub CLI (`gh`) for PR creation
- (Optional) Android emulator for mobile tests

## Output Structure

```
artifacts/
 desktop/
    <platform>/
        *.AppImage (Linux)
        *.dmg (macOS)
        *.exe (Windows)
 mobile/
    android/app/build/outputs/
 reports/
     playwright-report/
     vitest-report/
```

## Troubleshooting

### Fake Backend Not Starting
```bash
## Check Docker is running
docker ps

## Check fake backend logs
docker compose -f gui/docker-compose.yml logs fake-cryprq

## Manually start
docker compose -f gui/docker-compose.yml up -d fake-cryprq
```

### Tests Failing
- Check fake backend is reachable: `curl http://localhost:9464/metrics`
- Review test reports in `artifacts/reports/`
- Check Playwright reports: `artifacts/reports/playwright-report/index.html`

### Build Failing
- Verify platform-specific build commands exist in `package.json`
- Check `Makefile` exists for Linux builds
- Review build logs in terminal output

### Mobile Tests Skipped
- Android emulator not available (expected - CI covers this)
- Detox not installed: `npm install -g detox-cli`
- Emulator not running: Start Android Studio emulator first

## Cleanup

After validation:

```bash
## Stop fake backends
docker compose -f gui/docker-compose.yml down
docker compose -f mobile/docker-compose.yml down

## Remove artifacts (optional)
rm -rf artifacts/
```

## Integration with CI

This script mirrors CI workflows:
- **Desktop**: Same tests as `.github/workflows/release.yml`
- **Mobile**: Same tests as `.github/workflows/mobile-ci.yml`

Use this script to validate locally before pushing to CI.

## Next Steps

After successful validation:
1. Review artifacts in `artifacts/desktop/<platform>/`
2. Review test reports in `artifacts/reports/`
3. When ready to ship: `./scripts/go-live.sh 1.1.0 && ./scripts/verify-release.sh`

