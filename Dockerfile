
<<<<<<< Updated upstream
# --- build stage ---------------------------------------------------------
FROM rust:1.83.0-alpine AS builder

# install a new-enough compiler
RUN echo '@edge http://dl-cdn.alpinelinux.org/alpine/edge/community' >> /etc/apk/repositories && \
    apk add --no-cache \
        musl-dev pkgconfig openssl-dev openssl-libs-static \
        gcc@edge g++@edge make linux-headers

# ----------  WORK-AROUND  -------------------------------------------------
# define the missing macro so pqclean code compiles
RUN mkdir -p /usr/local/include && \
    echo '#define __GNUC_PREREQ(maj,min) 1' > /usr/local/include/compat.h
ENV CFLAGS="-include /usr/local/include/compat.h"
# --------------------------------------------------------------------------
=======
FROM messense/rust-musl-cross:x86_64-musl AS builder


## messense/rust-musl-cross already includes musl cross-compiler and build tools
>>>>>>> Stashed changes

WORKDIR /build
COPY . .
RUN cargo build --release -p cryprq

<<<<<<< Updated upstream
FROM alpine:3.21
RUN apk add --no-cache ca-certificates
COPY --from=builder /build/target/release/cryprq /usr/local/bin/cryprq
ENTRYPOINT ["cryprq"]
=======
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

    # Ensure musl target is installed
    RUN rustup target add x86_64-unknown-linux-musl

    RUN cargo build --release --target x86_64-unknown-linux-musl -p cryprq

    # Skipping strip step; musl binary is already optimized

# Runtime stage (minimal)
FROM alpine:3.19
RUN apk add --no-cache ca-certificates
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/cryprq /usr/local/bin/cryprq
ENTRYPOINT ["/usr/local/bin/cryprq"]
>>>>>>> Stashed changes
