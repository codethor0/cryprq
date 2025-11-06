# Reproducible Builds

CrypRQ binaries are byte-for-byte reproducible using either Nix or Docker.

## Prerequisites
- Git with GPG verification enabled
- SHA256 checksum tools
- Either: Nix (recommended) OR Docker

## Quick Verification

### Method 1: Nix Shell (One-Liner)
```bash
nix-shell --pure -p rustc cargo --run "cargo build --release --locked" && sha256sum target/release/cryprq
```

**Expected Output:**
```
[HASH] target/release/cryprq
```

### Method 2: Docker Reproducer
```bash
docker build -t cryprq-reproducible - <<'EOF'
FROM debian:bookworm-slim
ENV RUST_VERSION=1.82.0
RUN apt-get update && apt-get install -y curl build-essential git
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /build
COPY . .
RUN cargo build --release --locked
CMD ["sha256sum", "target/release/cryprq"]
EOF

docker run --rm cryprq-reproducible
```

## Full Reproducible Build Process

### Step 1: Clone with Verification
```bash
# Enable GPG signature verification
git config --global commit.gpgSign true
git config --global tag.gpgSign true

# Clone repository
git clone https://github.com/codethor0/cryprq.git
cd cryprq

# Verify latest commit signature
git verify-commit HEAD

# Checkout specific release tag
git checkout v0.1.0
git verify-tag v0.1.0
```

### Step 2: Verify rust-toolchain.toml
```bash
sha256sum rust-toolchain.toml
# Expected: [HASH_TO_BE_UPDATED]
```

### Step 3: Build with Nix
```bash
# Create shell.nix if not present
cat > shell.nix <<'EOF'
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    openssl
  ];
  
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  
  shellHook = ''
    export CARGO_HOME=$PWD/.cargo
    export RUSTUP_HOME=$PWD/.rustup
  '';
}
EOF

# Enter pure Nix shell
nix-shell --pure

# Build with locked dependencies
cargo build --release --locked

# Exit shell
exit

# Verify checksum
sha256sum target/release/cryprq
```

### Step 4: Compare with Official Release
```bash
# Download official release
gh release download v0.1.0 -p checksums.txt

# Compare checksums
sha256sum target/release/cryprq | cut -d' ' -f1 > local.sum
grep "cryprq-x86_64" checksums.txt | cut -d' ' -f1 > official.sum
diff local.sum official.sum && echo "✓ Build is reproducible"
```

## Docker Method (Detailed)

### Dockerfile.reproducible
```dockerfile
FROM debian:bookworm-slim

# Pin Rust version
ENV RUST_VERSION=1.82.0
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust with pinned version
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain ${RUST_VERSION} --profile minimal --component clippy rustfmt

# Set working directory
WORKDIR /build

# Copy project files
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crypto ./crypto
COPY p2p ./p2p
COPY node ./node
COPY cli ./cli

# Build with locked dependencies
RUN cargo build --release --locked

# Generate checksum
RUN sha256sum target/release/cryprq > /checksum.txt

# Default command
CMD ["cat", "/checksum.txt"]
```

### Build & Verify
```bash
# Build Docker image
docker build -f Dockerfile.reproducible -t cryprq-reproducible .

# Extract binary and checksum
docker run --rm cryprq-reproducible cat /checksum.txt
docker cp $(docker create cryprq-reproducible):/build/target/release/cryprq ./cryprq-docker

# Verify
sha256sum cryprq-docker
```

## Troubleshooting

### Different checksums?
1. Verify Rust version: `rustc --version` (must be 1.82.0)
2. Verify clean build: `cargo clean && cargo build --release --locked`
3. Check for local patches: `git diff HEAD`
4. Verify Cargo.lock: `git diff Cargo.lock`

### Nix build fails?
```bash
# Update Nix channels
nix-channel --update

# Clear Nix store
nix-collect-garbage -d

# Retry with verbose output
nix-shell --pure --show-trace
```

### Docker build fails?
```bash
# Clean Docker build cache
docker system prune -a

# Rebuild without cache
docker build --no-cache -f Dockerfile.reproducible -t cryprq-reproducible .
```

## Verification Checklist
- [ ] Git commit signature verified
- [ ] Git tag signature verified
- [ ] rust-toolchain.toml checksum matches
- [ ] Cargo.lock unchanged
- [ ] Build completed with --locked flag
- [ ] Binary checksum matches official release
- [ ] No local modifications (git status clean)

## References
- [Reproducible Builds Project](https://reproducible-builds.org/)
- [Nix Package Manager](https://nixos.org/manual/nix/stable/)
- [Rust Toolchain File](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file)
