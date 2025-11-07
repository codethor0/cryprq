
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

WORKDIR /build
COPY . .
RUN cargo build --release -p cryprq

FROM alpine:3.21
RUN apk add --no-cache ca-certificates
COPY --from=builder /build/target/release/cryprq /usr/local/bin/cryprq
ENTRYPOINT ["cryprq"]
