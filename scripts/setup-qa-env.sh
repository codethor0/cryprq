#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Setup QA environment matrix
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "QA Environment Setup"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Install Rust toolchains
echo "Installing Rust toolchains..."
rustup toolchain install 1.83.0 stable beta nightly --profile minimal
rustup default 1.83.0

# Set override for this project
rustup override set 1.83.0

# Install components
echo "Installing Rust components..."
rustup component add rustfmt clippy --toolchain 1.83.0
rustup component add rustfmt clippy --toolchain stable
rustup component add miri --toolchain nightly

# Install cargo tools
echo "Installing cargo tools..."
cargo install cargo-fuzz cargo-audit cargo-deny cargo-geiger cargo-llvm-cov cargo-vet || true

# Setup targets
echo "Setting up targets..."
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Create RUSTFLAGS configs
mkdir -p .cargo

cat > .cargo/config.toml << 'EOF'
[target.'cfg(all())']
rustflags = [
    "-C", "lto=thin",
    "-C", "opt-level=3",
    "-C", "codegen-units=1",
]

[target.'cfg(not(debug_assertions))']
rustflags = [
    "-C", "lto=fat",
    "-C", "opt-level=3",
    "-C", "codegen-units=1",
    "-C", "strip=symbols",
]

[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
EOF

echo ""
echo "✅ QA environment setup complete"
echo ""
echo "Toolchains: 1.83.0 (default), stable, beta, nightly"
echo "Components: rustfmt, clippy, miri"
echo "Targets: Linux (glibc, musl), macOS (x86_64, aarch64)"
echo "Cargo tools: fuzz, audit, deny, geiger, llvm-cov, vet"

