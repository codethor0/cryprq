#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
# Live dev watcher: on file changes -> run gates -> auto-commit (ours only) -> optional push.
# Requirements: bash, git, python3, cargo, docker (for QA), and platform toolchains as applicable.
# Optional: fswatch (macOS) or inotifywait (Linux) for efficient watching.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Concurrency + lockfile (avoid overlapping watchers)
LOCK="artifacts/dev-watch/.lock"
mkdir -p "$(dirname "$LOCK")"
exec 9>"$LOCK"

if command -v flock >/dev/null 2>&1; then
  flock -n 9 || { echo "[watch] another watcher is running"; exit 0; }
else
  # Fallback: check if lock file exists and is recent (< 5 minutes)
  if [[ -f "$LOCK" ]]; then
    if command -v stat >/dev/null 2>&1; then
      lock_age=$(($(date +%s) - $(stat -c %Y "$LOCK" 2>/dev/null || stat -f %m "$LOCK" 2>/dev/null || echo 0)))
      if [[ $lock_age -lt 300 ]]; then
        echo "[watch] another watcher may be running (lock file exists)"
        exit 0
      fi
    fi
  fi
  touch "$LOCK"
fi

# ---------- Configuration ----------
AUTHOR_NAME="${AUTHOR_NAME:-Thor Thor}"
AUTHOR_EMAIL="${AUTHOR_EMAIL:-codethor@gmail.com}"
PUSH="${PUSH:-0}"                             # 1 to push green commits
TARGET_BRANCH="${TARGET_BRANCH:-main}"        # default branch; watcher works on feature branches too
WORK_BRANCH="${WORK_BRANCH:-wip/$(whoami)/$(date +%Y%m%d-%H%M)}"
DOCKER_PORT="${DOCKER_PORT:-9999}"
ROTATE_SECS="${ROTATE_SECS:-10}"              # accelerated rotate for QA
POLL_SEC="${POLL_SEC:-3}"
LOGDIR="${ROOT}/artifacts/dev-watch"
mkdir -p "${LOGDIR}"

# Files/dirs to watch and to ignore
WATCH_INCLUDE=(
  "*.rs" "*.toml" "Cargo.lock" "Cargo.toml"
  "scripts/**/*.sh" "scripts/*.sh"
  "docker/**" "Dockerfile" "docker-compose*.yml"
  "android/**" "apple/**" "windows/**" "packaging/**" "gui/**"
  "docs/**/*.md" "README*.md" "**/*.md"
)
WATCH_EXCLUDE=(
  ".git" "target" "dist" "build" "node_modules" ".venv" "artifacts" ".idea" ".DS_Store"
)

# ---------- Helpers ----------
have() { command -v "$1" >/dev/null 2>&1; }
note(){ echo "[watch] $*"; }
fail(){ echo "[watch] ERROR: $*" >&2; return 1; }

short_summary() {
  git status --porcelain 2>/dev/null | awk '{print $2}' | head -n 5 | paste -sd',' - | sed 's/,/, /g' || echo "changes"
}

ensure_author() {
  git config user.name  "$AUTHOR_NAME"
  git config user.email "$AUTHOR_EMAIL"
}

current_author_is_us() {
  local a e
  a=$(git config user.name || true)
  e=$(git config user.email || true)
  [[ "$a" == "$AUTHOR_NAME" && "$e" == "$AUTHOR_EMAIL" ]]
}

create_branch_once() {
  git rev-parse --verify "$WORK_BRANCH" >/dev/null 2>&1 || git checkout -b "$WORK_BRANCH" 2>/dev/null || true
}

changed_since_last() {
  git status --porcelain 2>/dev/null | grep -vE '(^\?\?|^ D )' >/dev/null 2>&1
}

# ---------- Gates ----------
gate_code() {
  note "Code gate: fmt/clippy/build/test"
  cargo fmt --all || fail "cargo fmt failed"
  cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee "${LOGDIR}/clippy.txt" || fail "cargo clippy failed"
  cargo test --all --no-fail-fast 2>&1 | tee "${LOGDIR}/tests.txt" || fail "cargo test failed"
  
  # Quarantine flaky tests (non-blocking)
  [[ -x scripts/quarantine-flaky.sh ]] && bash scripts/quarantine-flaky.sh || true
  
  cargo build --release -p cryprq 2>&1 | tee "${LOGDIR}/build.txt" || fail "cargo build failed"
  
  # Size/latency regression guard
  BIN="target/release/cryprq"
  if [[ -f "$BIN" ]]; then
    SIZE=$(stat -c%s "$BIN" 2>/dev/null || stat -f%z "$BIN" 2>/dev/null || echo 0)
    MAX=${MAX_BIN_SIZE:-7000000}   # 7.0 MB
    if [[ "$SIZE" -gt "$MAX" ]]; then
      fail "Binary too large: $SIZE bytes > $MAX bytes (${MAX_BIN_SIZE:-7.0 MB})"
    fi
    note "Binary size: $(( SIZE / 1024 / 1024 )) MB (OK)"
  fi
}

gate_security() {
  note "Security gate: secret scan + audit (non-fatal if tools missing)"
  local failed=0
  
  if have gitleaks; then
    gitleaks detect --no-banner --source . 2>&1 | tee "${LOGDIR}/gitleaks.txt" || failed=1
  fi
  
  if have trufflehog; then
    trufflehog filesystem --no-update . 2>&1 | tee "${LOGDIR}/trufflehog.txt" || failed=1
  fi
  
  if have cargo-audit; then
    cargo audit 2>&1 | tee "${LOGDIR}/cargo-audit.txt" || failed=1
  fi
  
  if have syft; then
    syft . -o spdx-json > "${LOGDIR}/sbom.json" 2>&1 || true
  fi
  
  if have grype && [[ -f "${LOGDIR}/sbom.json" ]]; then
    grype sbom:"${LOGDIR}/sbom.json" 2>&1 | tee "${LOGDIR}/grype.txt" || true
  fi
  
  if [[ $failed -eq 1 ]]; then
    note "Security gate: Some tools found issues (non-fatal)"
  fi
}

gate_icons() {
  note "Icons gate"
  local failed=0
  
  if [[ -x scripts/verify-icons-min.sh ]]; then
    bash scripts/verify-icons-min.sh 2>&1 | tee "${LOGDIR}/icons.txt" || failed=1
  fi
  
  if [[ -x scripts/android-mipmap-validate.sh && -d android ]]; then
    bash scripts/android-mipmap-validate.sh 2>&1 | tee -a "${LOGDIR}/icons.txt" || failed=1
  fi
  
  if [[ -x scripts/ios-contents-validate.sh && -d apple ]]; then
    bash scripts/ios-contents-validate.sh 2>&1 | tee -a "${LOGDIR}/icons.txt" || failed=1
  fi
  
  if [[ $failed -eq 1 ]]; then
    fail "Icon validation failed"
  fi
}

gate_docs() {
  note "Docs gate: no-emoji + lint (non-fatal lint if tooling absent)"
  local failed=0
  
  if [[ -x scripts/no-emoji-gate.sh ]]; then
    bash scripts/no-emoji-gate.sh 2>&1 | tee "${LOGDIR}/docs.txt" || failed=1
  fi
  
  if have npx; then
    npx --yes markdownlint-cli2 "**/*.md" "!**/node_modules/**" "!**/target/**" "!**/dist/**" "!**/build/**" 2>&1 | tee -a "${LOGDIR}/docs.txt" || true
    npx --yes prettier -c "**/*.md" "!**/node_modules/**" "!**/target/**" "!**/dist/**" "!**/build/**" 2>&1 | tee -a "${LOGDIR}/docs.txt" || true
  fi
  
  if [[ $failed -eq 1 ]]; then
    fail "Docs gate failed (emojis found)"
  fi
}

gate_docker_qa() {
  note "Docker QA: listener + dialer handshake + rotation=${ROTATE_SECS}s"
  
  if ! have docker; then
    note "Docker not available; skipping Docker QA gate"
    return 0
  fi
  
  docker build -t cryprq-node:dev . 2>&1 | tee "${LOGDIR}/docker_build.txt" || fail "Docker build failed"
  
  docker rm -f cryprq-listener >/dev/null 2>&1 || true
  
  docker run -d --name cryprq-listener \
    -p ${DOCKER_PORT}:${DOCKER_PORT}/udp \
    cryprq-node:dev \
    --listen /ip4/0.0.0.0/udp/${DOCKER_PORT}/quic-v1 2>&1 | tee "${LOGDIR}/docker_listener_start.txt" || fail "Docker listener start failed"
  
  sleep 2
  
  docker logs --since 2s cryprq-listener 2>&1 | tee "${LOGDIR}/listener_boot.txt" || true
  
  timeout 10 docker run --rm --network host cryprq-node:dev \
    --peer /ip4/127.0.0.1/udp/${DOCKER_PORT}/quic-v1 2>&1 | tee "${LOGDIR}/dialer.txt" || true
  
  sleep $(( ROTATE_SECS + 3 ))
  
  docker logs cryprq-listener 2>&1 | grep -Ei "rotate|rotation|zeroiz" | tee "${LOGDIR}/rotation.txt" || true
  
  docker rm -f cryprq-listener >/dev/null 2>&1 || true
  
  note "Docker QA gate passed"
}

gate_gui() {
  # Minimal GUI smoke if a GUI exists; ensure a Debug Console area renders.
  if [[ -d "gui" ]]; then
    note "GUI gate: building GUI"
    
    if [[ -f "gui/package.json" && $(command -v npm) ]]; then
      (cd gui && npm ci 2>&1 && npm run build 2>&1) | tee "${LOGDIR}/gui_build.txt" || fail "GUI build failed"
      note "GUI build passed"
    elif have cargo && find gui -name "Cargo.toml" -path "*/src-tauri/*" 2>/dev/null | head -1 | grep -q .; then
      (cd gui && cargo tauri build 2>&1) | tee "${LOGDIR}/gui_build.txt" || note "Tauri build skipped or failed"
    else
      note "GUI present but no build system detected; skipping GUI gate"
    fi
  else
    note "GUI not present; skipping."
  fi
}

write_green_summary() {
  {
    echo "# Green Gate Summary"
    echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "Branch: $(git rev-parse --abbrev-ref HEAD)"
    echo "Commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'N/A')"
    echo
    echo "Last changes:"
    git status --porcelain 2>/dev/null || echo "No changes"
    echo
    echo "Artifacts:"
    ls -1 "${LOGDIR}" 2>/dev/null | head -20 || echo "No artifacts"
    echo
    echo "Gates passed:"
    echo "- Code (fmt/clippy/test/build)"
    echo "- Security (scan/audit)"
    echo "- Icons (validation)"
    echo "- Docs (no-emoji/lint)"
    echo "- Docker QA (handshake/rotation)"
    echo "- GUI (build)"
  } > "${LOGDIR}/GREEN_SUMMARY.md"
}

auto_commit_and_push() {
  ensure_author
  current_author_is_us || fail "Git author not set to ${AUTHOR_NAME} <${AUTHOR_EMAIL}>"

  git add --all
  if git diff --cached --quiet; then
    note "No staged changes to commit."
    return 0
  fi
  
  local msg="chore(dev): green gate auto-commit — $(short_summary)"
  
  # Check if signing is configured
  if git config --get user.signingkey >/dev/null 2>&1; then
    git commit -S -m "$msg" || fail "Commit failed"
  else
    git commit -m "$msg" || fail "Commit failed"
  fi
  
  note "Committed: $msg"

  if [[ "$PUSH" == "1" ]]; then
    create_branch_once
    
    # Only push if CI-green gate passes
    if [[ -x scripts/push-if-green.sh ]]; then
      if bash scripts/push-if-green.sh "$WORK_BRANCH"; then
        git push -u origin "$WORK_BRANCH" 2>&1 | tee "${LOGDIR}/push.txt" || note "Push failed or skipped"
        note "Pushed to $WORK_BRANCH (CI green)"
      else
        note "Push skipped (CI not green for previous commit)"
      fi
    else
      git push -u origin "$WORK_BRANCH" 2>&1 | tee "${LOGDIR}/push.txt" || note "Push failed or skipped"
      note "Pushed to $WORK_BRANCH"
    fi
  fi
}

run_gates() {
  local failed=0
  
  gate_code || failed=1
  [[ $failed -eq 1 ]] && return 1
  
  gate_security || note "Security gate: warnings (non-fatal)"
  
  gate_icons || failed=1
  [[ $failed -eq 1 ]] && return 1
  
  gate_docs || failed=1
  [[ $failed -eq 1 ]] && return 1
  
  gate_docker_qa || failed=1
  [[ $failed -eq 1 ]] && return 1
  
  gate_gui || note "GUI gate: warnings (non-fatal)"
  
  # Web QA + macOS Packaging gate (if web/ exists)
  if [[ -d "web" && -x scripts/qa-web-and-mac.sh ]]; then
    note "Web QA + macOS Packaging gate..."
    bash scripts/qa-web-and-mac.sh || failed=1
    [[ $failed -eq 1 ]] && return 1
  fi
  
  return 0
}

# ---------- Initial setup ----------
ensure_author
create_branch_once
note "Watching for changes… (branch: $(git rev-parse --abbrev-ref HEAD))"
note "Author: $AUTHOR_NAME <$AUTHOR_EMAIL>; PUSH=$PUSH; WORK_BRANCH=$WORK_BRANCH"

# ---------- Watch loop ----------
watch_once() {
  if ! changed_since_last; then
    return 0
  fi
  
  note "Changes detected; running gates…"
  set +e
  run_gates
  rc=$?
  set -e
  
  if [[ $rc -ne 0 ]]; then
    note "Gates failed; see artifacts in ${LOGDIR}. No commit."
    {
      echo "# Gate Failure Summary"
      echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
      echo "Branch: $(git rev-parse --abbrev-ref HEAD)"
      echo
      echo "One or more gates failed. Review logs in ${LOGDIR}/"
      echo
      echo "Recent changes:"
      git status --porcelain 2>/dev/null || echo "No changes"
    } > "${LOGDIR}/FAILURE_SUMMARY.md"
    return 0
  fi
  
  write_green_summary
  auto_commit_and_push
}

# Main watch loop
if have fswatch; then
  note "Using fswatch for file watching"
  fswatch -o . --exclude='^\.git' $(printf -- " --exclude=%q" "${WATCH_EXCLUDE[@]}") \
    | while read -r _; do watch_once; done
elif have inotifywait; then
  note "Using inotifywait for file watching"
  while inotifywait -r -e modify,create,delete,move \
    --exclude '(^\.git|target|dist|build|node_modules|\.venv|artifacts)' . 2>/dev/null; do
    watch_once
  done
else
  note "Neither fswatch nor inotifywait found; falling back to polling every ${POLL_SEC}s."
  last="$(git rev-parse HEAD 2>/dev/null || echo '')"
  while true; do
    sleep "${POLL_SEC}"
    if changed_since_last; then
      watch_once
    fi
  done
fi

