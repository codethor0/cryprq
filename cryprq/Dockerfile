# ---- builder ----
FROM messense/rust-musl-cross:x86_64-musl AS builder
WORKDIR /build

# Minimal, cache-friendly copies
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY cli/Cargo.toml ./cli/
COPY crypto/Cargo.toml ./crypto/
COPY node/Cargo.toml ./node/
COPY p2p/Cargo.toml ./p2p/
COPY cli/src ./cli/src
COPY crypto/src ./crypto/src
COPY node/src ./node/src
COPY p2p/src ./p2p/src

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl -p cryprq

# ---- runtime ----
FROM alpine:3.19
RUN apk add --no-cache ca-certificates
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/cryprq /usr/local/bin/cryprq
ENTRYPOINT ["/usr/local/bin/cryprq"]