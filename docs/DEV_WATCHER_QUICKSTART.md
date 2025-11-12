# Development Watcher Quick Start

## One-Liner (Push Enabled)

```bash
AUTHOR_NAME="Thor Thor" AUTHOR_EMAIL="codethor@gmail.com" PUSH=1 MAX_BIN_SIZE=7000000 ROTATE_SECS=10 bash scripts/dev-watch.sh
```

## One-Liner (No Push)

```bash
AUTHOR_NAME="Thor Thor" AUTHOR_EMAIL="codethor@gmail.com" PUSH=0 bash scripts/dev-watch.sh
```

## What It Does

1. **Watches** your repository for file changes
2. **Runs 6 green gates** in sequence (fail fast):
   - Code: fmt/clippy/test/build + size guard + quarantine
   - Security: secret scans/audit/SBOM
   - Icons: validation
   - Docs: no-emoji/lint
   - Docker QA: handshake/rotation test
   - GUI: build with safe debug console
3. **Auto-commits** only when all gates pass
4. **Pushes** only if CI-green gate passes (when `PUSH=1`)

## Configuration

### Environment Variables

- `AUTHOR_NAME`: Git author name (default: "Thor Thor")
- `AUTHOR_EMAIL`: Git author email (default: "codethor@gmail.com")
- `PUSH`: Push to wip branch (default: 0)
- `WORK_BRANCH`: Work branch name (default: `wip/$(whoami)/$(date +%Y%m%d-%H%M)`)
- `DOCKER_PORT`: Docker listener port (default: 9999)
- `ROTATE_SECS`: Accelerated rotation for QA (default: 10)
- `POLL_SEC`: Polling interval if no file watcher (default: 3)
- `MAX_BIN_SIZE`: Maximum binary size in bytes (default: 7000000 = 7 MB)

### Example: Custom Binary Size

```bash
MAX_BIN_SIZE=10000000 bash scripts/dev-watch.sh  # 10 MB limit
```

### Example: Faster Rotation Testing

```bash
ROTATE_SECS=5 bash scripts/dev-watch.sh  # 5 second rotation
```

## Output

### Success

- Commits with message: `chore(dev): green gate auto-commit — <summary>`
- Writes `artifacts/dev-watch/GREEN_SUMMARY.md`
- Optionally pushes to `wip/<user>/<timestamp>` branch (if CI green)

### Failure

- Does not commit
- Writes `artifacts/dev-watch/FAILURE_SUMMARY.md`
- Logs all gate outputs to `artifacts/dev-watch/`

### Flaky Tests

- Documented in `artifacts/dev-watch/QUARANTINED_TESTS.txt`
- Does not block dev loop
- Tracked for nightly triage

## Requirements

- `bash`, `git`, `cargo`, `docker`
- Optional: `fswatch` (macOS) or `inotifywait` (Linux) for efficient watching
- Optional: `gh` CLI for CI-green gate (install: `brew install gh` or `apt-get install gh`)

## Troubleshooting

### Lock file prevents watcher

```bash
rm artifacts/dev-watch/.lock
```

### CI-green gate always fails

- Ensure `gh` CLI is installed and authenticated: `gh auth login`
- Check GitHub Actions workflows are configured

### Binary size exceeded

- Review dependencies and features
- Adjust `MAX_BIN_SIZE` if legitimate growth

## See Also

- `docs/DEV_WATCHER_HARDENING.md` - Complete hardening documentation
- `scripts/dev-watch.sh` - Main watcher script
- `scripts/push-if-green.sh` - CI-green gate
- `scripts/quarantine-flaky.sh` - Flaky test tracker

