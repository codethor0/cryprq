# Development Watcher Service Setup

Run the dev-watcher as a background service on your development machine.

## Quick Setup

### 1. Create Environment File

```bash
cp .env.devwatch.example .env.devwatch
# Edit .env.devwatch with your settings
```

### 2. Choose Your Platform

**Linux (systemd):**
```bash
bash scripts/setup-service.sh systemd
systemctl --user daemon-reload
systemctl --user enable dev-watch.service
systemctl --user start dev-watch.service
```

**macOS (launchd):**
```bash
bash scripts/setup-service.sh launchd
launchctl load ~/Library/LaunchAgents/com.cryrpq.devwatch.plist
launchctl start com.cryrpq.devwatch
```

**tmux (Linux/macOS):**
```bash
tmux new -s devwatch 'set -a; source ./.env.devwatch 2>/dev/null || true; set +a; bash scripts/dev-watch.sh'
```

## Manual Setup

### Linux (systemd)

1. Copy service file:
```bash
mkdir -p ~/.config/systemd/user
cp scripts/dev-watch.service.example ~/.config/systemd/user/dev-watch.service
```

2. Edit `~/.config/systemd/user/dev-watch.service`:
   - Replace `/path/to/your/repo` with your actual repository path
   - Ensure `.env.devwatch` path is correct

3. Enable and start:
```bash
systemctl --user daemon-reload
systemctl --user enable dev-watch.service
systemctl --user start dev-watch.service
```

4. Check status:
```bash
systemctl --user status dev-watch.service
```

5. View logs:
```bash
journalctl --user -u dev-watch.service -f
```

### macOS (launchd)

1. Copy plist file:
```bash
cp scripts/com.cryrpq.devwatch.plist.example ~/Library/LaunchAgents/com.cryrpq.devwatch.plist
```

2. Edit `~/Library/LaunchAgents/com.cryrpq.devwatch.plist`:
   - Replace `/Users/YOU/path/to/repo` with your actual repository path

3. Load and start:
```bash
launchctl unload ~/Library/LaunchAgents/com.cryrpq.devwatch.plist 2>/dev/null || true
launchctl load ~/Library/LaunchAgents/com.cryrpq.devwatch.plist
launchctl start com.cryrpq.devwatch
```

4. Check status:
```bash
launchctl list | grep cryrpq
```

5. View logs:
```bash
tail -f artifacts/dev-watch/launchd.out
tail -f artifacts/dev-watch/launchd.err
```

## Environment Configuration

Edit `.env.devwatch` to customize:

```bash
AUTHOR_NAME="Thor Thor"
AUTHOR_EMAIL="codethor@gmail.com"
PUSH=1
MAX_BIN_SIZE=7000000
ROTATE_SECS=10
WORK_BRANCH="wip/thor-iso/$(date +%Y%m%d-%H%M)"
```

## Safe Rollback

If something slips through the gates:

```bash
# Undo last green auto-commit (keeps changes staged)
bash scripts/rollback-last-commit.sh

# Or manually:
git reset --soft HEAD~1  # Keep changes staged
git reset --hard HEAD~1   # Discard changes (careful!)
```

## Service Management

### systemd (Linux)

```bash
# Start
systemctl --user start dev-watch.service

# Stop
systemctl --user stop dev-watch.service

# Restart
systemctl --user restart dev-watch.service

# Disable (stop auto-start)
systemctl --user disable dev-watch.service

# Status
systemctl --user status dev-watch.service

# Logs
journalctl --user -u dev-watch.service -f
```

### launchd (macOS)

```bash
# Start
launchctl start com.cryrpq.devwatch

# Stop
launchctl stop com.cryrpq.devwatch

# Unload (remove from auto-start)
launchctl unload ~/Library/LaunchAgents/com.cryrpq.devwatch.plist

# Reload (after editing plist)
launchctl unload ~/Library/LaunchAgents/com.cryrpq.devwatch.plist
launchctl load ~/Library/LaunchAgents/com.cryrpq.devwatch.plist

# Status
launchctl list | grep cryrpq
```

## Nightly Hard Fail Job

A GitHub Actions workflow (`.github/workflows/nightly-hard-fail.yml`) runs daily at 2 AM UTC to:

- **Fail on quarantined tests** - Ensures flaky tests are fixed
- **Re-run all tests** - Catches any regressions
- **Check binary size** - Enforces size limits
- **Security audit** - Blocks on High/Critical vulnerabilities
- **SBOM scan** - Full vulnerability scan with Grype

This keeps the daytime dev loop fast while ensuring rigor overnight.

## Troubleshooting

### Service won't start

1. Check logs:
   - systemd: `journalctl --user -u dev-watch.service`
   - launchd: `tail -f artifacts/dev-watch/launchd.err`

2. Verify paths in service file are correct

3. Ensure `.env.devwatch` exists and is readable

4. Check lockfile: `rm artifacts/dev-watch/.lock` if stale

### Service keeps restarting

- Check for errors in logs
- Verify all required tools are installed (cargo, docker, etc.)
- Ensure repository is accessible

### Changes not being committed

- Check that author is set correctly in `.env.devwatch`
- Verify gates are passing (check `artifacts/dev-watch/FAILURE_SUMMARY.md`)
- Ensure lockfile isn't stale

## See Also

- `docs/DEV_WATCHER_QUICKSTART.md` - Quick start guide
- `docs/DEV_WATCHER_HARDENING.md` - Hardening documentation
- `scripts/dev-watch.sh` - Main watcher script

