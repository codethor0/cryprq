# Development Watcher Hardening

This document describes the hardening add-ons that make the dev-watcher workflow bulletproof and self-healing.

## Hardening Components

### 1. CI-Green Gate (Server-Side Safety)

**Script**: `scripts/push-if-green.sh`

**Purpose**: Prevents pushing commits unless the previous commit has green CI status

**How it works**:
- Uses GitHub CLI (`gh`) to check CI status
- Polls the last 20 workflow runs for the current branch
- Waits up to 2 minutes (12 polls × 10 seconds) for CI to complete
- Only allows push if CI is green or skipped

**Integration**: Called automatically before push in `dev-watch.sh` when `PUSH=1`

**Usage**:
```bash
bash scripts/push-if-green.sh <branch-name>
```

### 2. Flaky Test Quarantine

**Script**: `scripts/quarantine-flaky.sh`

**Purpose**: Tracks failing tests without blocking the dev loop

**How it works**:
- Captures failed tests from the last `cargo test` run
- Documents them in `artifacts/dev-watch/QUARANTINED_TESTS.txt`
- Does not mute tests automatically (leaves to nightly CI)
- Keeps dev loop green while making failures visible

**Integration**: Runs automatically after `cargo test` in the code gate

**Output**: `artifacts/dev-watch/QUARANTINED_TESTS.txt` with:
- Date and count of failing tests
- List of test names
- Note that they're documented for nightly triage

### 3. Auto-Changelog + Version Bump

**Script**: `scripts/release-notes.sh`

**Purpose**: Generates CHANGELOG.md from commit messages and creates release tags

**How it works**:
- Extracts commits since last tag (or all commits if no tag)
- Formats as conventional commits
- Prepends to existing CHANGELOG.md
- Creates annotated tag

**Usage**:
```bash
bash scripts/release-notes.sh v1.2.3
```

**Integration**: Call manually after green commit, or from release scripts

### 4. Concurrency Lockfile

**Location**: Top of `scripts/dev-watch.sh`

**Purpose**: Prevents multiple watchers from running simultaneously

**How it works**:
- Uses `flock` (file locking) if available
- Falls back to lock file age check (< 5 minutes)
- Exits gracefully if another watcher is detected

**Lock file**: `artifacts/dev-watch/.lock`

### 5. Binary Size Regression Guard

**Location**: Inside `gate_code()` in `scripts/dev-watch.sh`

**Purpose**: Fails the gate if binary exceeds size limit

**How it works**:
- Checks `target/release/cryprq` size after build
- Default limit: 7.0 MB (`MAX_BIN_SIZE` env var)
- Fails fast with clear error message

**Configuration**:
```bash
MAX_BIN_SIZE=10000000 bash scripts/dev-watch.sh  # 10 MB limit
```

### 6. GUI Debug Console Privacy Guard

**File**: `gui/src/lib/safeLog.ts`

**Purpose**: Redacts secrets and sensitive data from debug console logs

**Patterns redacted**:
- Common secret keywords (private, secret, token, bearer, api_key)
- Cryptographic keys (sk, pk, seed, nonce)
- Long hex/base64 strings (32+ chars)
- Email addresses
- Multiaddr with embedded keys

**Usage**:
```typescript
import { safeLog } from '@/lib/safeLog'

const safeMessage = safeLog(rawMessage)
```

**Integration**: Automatically applied in `DebugConsole.tsx` to all log messages

### 7. Updated Master Prompt

**Role**: Development Orchestrator (Auto-Commit)

**Objective**: Maintain continuous cycle with server-side CI validation

**Key additions**:
- Only push if CI-green gate passes
- Quarantine flaky tests (document, don't mute)
- Redact secrets in GUI debug console
- Binary size regression guard
- Concurrency protection

## Workflow Flow

1. **File Change Detected** → Watcher triggers
2. **Lock Check** → Ensure no other watcher running
3. **Code Gate** → fmt, clippy, test, build, size check
4. **Quarantine** → Document failing tests (non-blocking)
5. **Security Gate** → Secret scan, audit, SBOM
6. **Icons Gate** → Validation
7. **Docs Gate** → No-emoji, lint
8. **Docker QA Gate** → Handshake + rotation test
9. **GUI Gate** → Build with debug console (redacted)
10. **All Green?** → Commit with author identity
11. **Push?** → Check CI-green gate → Push if green
12. **Summary** → Write GREEN_SUMMARY.md or FAILURE_SUMMARY.md

## Configuration

### Environment Variables

- `AUTHOR_NAME`: Git author name (default: "Thor Thor")
- `AUTHOR_EMAIL`: Git author email (default: "codethor@gmail.com")
- `PUSH`: Push to wip branch (default: 0)
- `WORK_BRANCH`: Work branch name (default: `wip/$(whoami)/$(date +%Y%m%d-%H%M)`)
- `DOCKER_PORT`: Docker listener port (default: 9999)
- `ROTATE_SECS`: Accelerated rotation for QA (default: 10)
- `POLL_SEC`: Polling interval if no file watcher (default: 3)
- `MAX_BIN_SIZE`: Maximum binary size in bytes (default: 7000000)

### GitHub CLI

The CI-green gate requires GitHub CLI (`gh`) to be installed and authenticated:

```bash
# Install gh CLI
brew install gh  # macOS
# or
apt-get install gh  # Linux

# Authenticate
gh auth login
```

## Acceptance Criteria

-  No red commits (all gates must pass)
-  Every pushed commit has passed all local gates
-  Push only happens if previous commit has green CI
-  GUI debug console redacts secrets automatically
-  Binary size stays within limits
-  Flaky tests are documented, not muted
-  Only one watcher runs at a time
-  Documentation remains emoji-free

## Troubleshooting

### CI-green gate always fails

- Ensure `gh` CLI is installed and authenticated
- Check that GitHub Actions workflows are configured
- Verify branch name matches CI branch

### Lock file prevents watcher

- Remove `artifacts/dev-watch/.lock` if watcher crashed
- Check for zombie watcher processes

### Binary size exceeded

- Review dependencies and features
- Consider stripping symbols or using LTO
- Adjust `MAX_BIN_SIZE` if legitimate growth

### Secrets still showing in debug console

- Check `safeLog.ts` patterns match your secret formats
- Add custom patterns if needed
- Verify `safeLog()` is called on all log messages

## Future Enhancements

- [ ] Per-gate timeout configuration
- [ ] Gate result caching (skip unchanged files)
- [ ] Webhook integration for CI status
- [ ] Metrics dashboard for gate performance
- [ ] Auto-retry flaky tests with exponential backoff

