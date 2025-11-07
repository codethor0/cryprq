#!/bin/bash
set -e

echo "Fixing Dockerfile and rebuilding container..."

# Write clean Dockerfile
cat <<'DOCKERFILE' > Dockerfile.reproducible
FROM debian:bookworm-slim

ENV RUST_VERSION=1.82.0
ENV PATH="/root/.cargo/bin:$PATH"

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl build-essential git pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain ${RUST_VERSION} --profile minimal

# Add components after installation
RUN rustup component add clippy rustfmt

# Copy source and build
COPY . /usr/src/cryprq
WORKDIR /usr/src/cryprq
RUN cargo build --release

CMD ["cargo", "test"]
DOCKERFILE

echo "Dockerfile fixed. Starting build..."
docker build -t cryprq-dev -f Dockerfile.reproducible .
echo "Build complete. Running tests..."
docker run --rm -it cryprq-dev cargo test || echo "Tests failed, but build succeeded."
