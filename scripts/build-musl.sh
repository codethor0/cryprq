#!/usr/bin/env bash
set -euo pipefail

# Reproducible musl build script for CrypRQ
# Produces ~4MB static binary with UPX compression

echo "=== CrypRQ Reproducible Build ==="
echo ""

# Check for required tools
command -v cargo >/dev/null 2>&1 || { echo "cargo not found. Install Rust toolchain."; exit 1; }

# Add musl target if not present
rustup target add x86_64-unknown-linux-musl 2>/dev/null || true

echo "Building for x86_64-unknown-linux-musl..."
echo ""

# Set reproducible build flags
export SOURCE_DATE_EPOCH=0
export RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-s -C codegen-units=1"

# Build with musl target
if command -v cargo-zigbuild >/dev/null 2>&1; then
    echo "Using cargo-zigbuild for reproducible build..."
    cargo zigbuild --release --target x86_64-unknown-linux-musl -p cryprq
else
    echo "Using cargo build (install cargo-zigbuild for better reproducibility)..."
    cargo build --release --target x86_64-unknown-linux-musl -p cryprq
fi

BINARY="target/x86_64-unknown-linux-musl/release/cryprq"

echo ""
echo "=== Binary Information ==="
file "$BINARY"
ls -lh "$BINARY"

# Strip symbols
if command -v strip >/dev/null 2>&1; then
    echo ""
    echo "Stripping symbols..."
    strip "$BINARY"
    ls -lh "$BINARY"
fi

# Optional UPX compression
if command -v upx >/dev/null 2>&1; then
    echo ""
    read -p "Compress with UPX? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cp "$BINARY" "${BINARY}.uncompressed"
        upx --best --lzma "$BINARY"
        echo ""
        echo "=== Compressed Binary ==="
        ls -lh "$BINARY"
        echo ""
        echo "Uncompressed backup: ${BINARY}.uncompressed"
    fi
else
    echo ""
    echo "UPX not found. Install for additional compression (optional)."
    echo "  brew install upx      # macOS"
    echo "  apt install upx-ucl   # Debian/Ubuntu"
fi

echo ""
echo "=== Build Complete ==="
echo "Binary: $BINARY"
echo ""
echo "Verify:"
echo "  sha256sum $BINARY"
echo ""
echo "Run:"
echo "  $BINARY --help"
