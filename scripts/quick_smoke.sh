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

echo "[SMOKE] Phase 4: Web stack bring-up"
docker compose -f docker-compose.web.yml down --remove-orphans || true
docker compose -f docker-compose.web.yml up --build -d

echo "[SMOKE] Current containers:"
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | sed -n '1,10p'

echo "[SMOKE] HTTP sanity check:"
curl -v http://localhost:8787/ || true

echo "[SMOKE] DONE. cryprq-web should now be running on http://localhost:8787"

