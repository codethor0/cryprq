# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# ---- builder ----
FROM rust:1.83 AS builder
WORKDIR /build

# Minimal, cache-friendly copies
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY cli/Cargo.toml ./cli/
COPY crypto/Cargo.toml ./crypto/
COPY node/Cargo.toml ./node/
COPY p2p/Cargo.toml ./p2p/
COPY core/Cargo.toml ./core/
COPY fuzz/Cargo.toml ./fuzz/
COPY cli/src ./cli/src
COPY crypto/src ./crypto/src
COPY node/src ./node/src
COPY p2p/src ./p2p/src
COPY core/src ./core/src
COPY third_party/if-watch ./third_party/if-watch

# Build optimized binary for Linux
RUN cargo build --release -p cryprq

# ---- runtime ----
FROM debian:bookworm-slim
LABEL org.opencontainers.image.title="CrypRQ"
LABEL org.opencontainers.image.description="Post-Quantum, Zero-Trust VPN"
LABEL org.opencontainers.image.vendor="CrypRQ"
LABEL org.opencontainers.image.logo="https://raw.githubusercontent.com/codethor0/cryprq/main/store-assets/icon_master_1024.png"
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        bash \
        curl \
        procps \
        iproute2 \
        iptables \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/cryprq /usr/local/bin/cryprq
ENTRYPOINT ["/usr/local/bin/cryprq"]
