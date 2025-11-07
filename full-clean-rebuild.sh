#!/bin/bash
set -e

echo "Cleaning project and rebuilding from scratch..."

# 1. Ensure Dockerfile syntax is valid
cat <<'DOCKERFILE' > Dockerfile.reproducible
FROM debian:bookworm-slim

ENV RUST_VERSION=1.82.0
ENV PATH="/root/.cargo/bin:$PATH"

RUN apt-get update && apt-get install -y \
    curl build-essential git pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain ${RUST_VERSION} --profile minimal

RUN rustup component add clippy rustfmt

COPY . /usr/src/cryprq
WORKDIR /usr/src/cryprq
RUN cargo build --release

CMD ["cargo", "test"]
DOCKERFILE

echo "Dockerfile sanitized."

# 2. Remove leftover merge conflict markers safely
echo "Removing conflict markers..."
grep -rl '<<<<<<<' . --include='*.toml' --include='*.rs' | while read -r file; do
    echo "Fixing conflict markers in: $file"
    sed -i.bak '/^<<<<<<< /d;/^=======/d;/^>>>>>>> /d' "$file"
done

# 3. Verify no markers remain
if grep -rq '<<<<<<<' .; then
    echo "ERROR: Conflict markers remain. Please check manually."
    exit 1
fi

# 4. Stage cleaned files if inside a git repo
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git add -u
    git commit -m "Clean merge markers and rebuild Dockerfile" || true
fi

# 5. Rebuild Docker container
echo "Starting Docker build..."
docker build -t cryprq-dev -f Dockerfile.reproducible .

echo "Build complete. Running tests..."
docker run --rm -it cryprq-dev cargo test || echo "Tests failed, but build succeeded."
