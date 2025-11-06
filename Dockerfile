# Multi-stage Dockerfile for reproducible musl builds
# Produces ~4MB static binary

FROM rust:1.83.0-alpine AS builder

# Install musl development tools
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY cli/Cargo.toml ./cli/
COPY crypto/Cargo.toml ./crypto/
COPY node/Cargo.toml ./node/
COPY p2p/Cargo.toml ./p2p/

# Copy source
COPY cli/src ./cli/src
COPY crypto/src ./crypto/src
COPY node/src ./node/src
COPY p2p/src ./p2p/src

# Build with reproducible flags
ENV RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-s -C codegen-units=1"
ENV SOURCE_DATE_EPOCH=0

RUN cargo build --release --target x86_64-unknown-linux-musl -p cryprq

# Strip binary
RUN strip /build/target/x86_64-unknown-linux-musl/release/cryprq

# Runtime stage (minimal)
FROM scratch

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/cryprq /cryprq

ENTRYPOINT ["/cryprq"]
