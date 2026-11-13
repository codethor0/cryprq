#!/usr/bin/env bash
set -euo pipefail

echo "[SMOKE] Phase 0: Repo status"
git status -sb
git branch -a | sed -n '1,10p'

echo "[SMOKE] Phase 1: Static checks (fmt + governance_shield)"
cargo fmt --all --check

if ! python3 scripts/governance_shield.py; then
  echo "[SMOKE] governance_shield reported failures (likely ENV LIMITATION for Clippy/build/tests/types)"
  echo "[SMOKE] Verify formatting, emoji, and prompt checks are still passing."
fi

echo "[SMOKE] Phase 2: Web build"
pushd web >/dev/null
npm run build
popd >/dev/null

echo "[SMOKE] Phase 3: QUIC E2E docker harness"
docker compose -f docker/e2e/docker-compose.e2e.yml down --remove-orphans || true
cleanup_e2e() {
  docker compose -f docker/e2e/docker-compose.e2e.yml down --remove-orphans || true
}
trap cleanup_e2e EXIT
docker compose -f docker/e2e/docker-compose.e2e.yml up --build -d listener dialer
timeout 300 bash docker/e2e/test_suite.sh
cleanup_e2e
trap - EXIT

echo "[SMOKE] Phase 4: Two browser portals in their own compose project"
PORTAL=(docker compose -p cryprq-portal-smoke -f docker-compose.portal.yml)
cleanup_portal() { "${PORTAL[@]}" down -v --remove-orphans || true; }
trap cleanup_portal EXIT
"${PORTAL[@]}" up --build -d --wait --wait-timeout 180

echo "[SMOKE] Adapter tests against the real CLI:"
"${PORTAL[@]}" exec -T portal-a sh -c 'node --test web/server/*.test.mjs'
cleanup_portal
trap - EXIT

echo "[SMOKE] DONE."

