#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="${REPO_ROOT:-$(pwd)}"
cd "$REPO_ROOT"

echo "[+] CrypRQ full validation starting"
echo "[+] Repo root: $REPO_ROOT"

########################################
# 1. Rust build / test / doc
########################################
echo "[1/4] Rust build/test/doc"

cargo build --release
echo "[+] cargo build --release OK"

cargo test
echo "[+] cargo test OK"

cargo doc --no-deps
echo "[+] cargo doc --no-deps OK"

########################################
# 2. Web build (if present)
########################################
if [ -f "web/package.json" ]; then
  echo "[2/4] Web build"
  pushd web > /dev/null
  npm install
  npm run build
  echo "[+] web build OK"
  popd > /dev/null
else
  echo "[2/4] No web/package.json found, skipping web build"
fi

########################################
# 3. Docker flows
########################################
echo "[3/4] Docker compose flows"

# You can customize or extend this list:
COMPOSE_FILES=(
  "docker-compose.web.yml"
  "docker-compose.vpn.yml"
)

for cf in "${COMPOSE_FILES[@]}"; do
  if [ -f "$cf" ]; then
    echo "[+] Bringing up stack via $cf"
    docker compose -f "$cf" up --build -d
    echo "[+] docker compose -f $cf up --build -d OK"

    echo "[+] docker compose -f $cf ps:"
    docker compose -f "$cf" ps

    echo "[+] Recent logs for $cf:"
    docker compose -f "$cf" logs --tail=80 || true
    
    echo "[+] Stopping stack via $cf"
    docker compose -f "$cf" down
  else
    echo "[+] $cf not found, skipping"
  fi
done

########################################
# 4. Summary / manual steps
########################################
echo "[4/4] Validation pipeline completed."
echo "Next manual checks:"
echo "  - Use the CLI to run a listener + dialer and transfer a test file."
echo "  - Use the web UI (if applicable) to establish a session and transfer data."
echo "  - Watch logs for handshake/tunnel events and confirm no unexpected errors."

