#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# One-shot bootstrap: pin toolchain & deps
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-release-$(date +%Y%m%d)/qa/bootstrap}"
mkdir -p "$ARTIFACT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "QA Bootstrap: Pin Toolchain & Dependencies"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Ensure Rust toolchains
echo "Installing Rust toolchains..."
rustup toolchain install stable beta nightly 1.83.0 --profile minimal || true
rustup default 1.83.0

# Read pinned toolchain if rust-toolchain.toml exists
if [ -f "rust-toolchain.toml" ]; then
    PINNED_TOOLCHAIN=$(grep "channel" rust-toolchain.toml | head -1 | sed 's/.*"\(.*\)".*/\1/' || echo "1.83.0")
    echo "Pinned toolchain from rust-toolchain.toml: $PINNED_TOOLCHAIN"
    rustup override set "$PINNED_TOOLCHAIN" || rustup override set 1.83.0
else
    echo "No rust-toolchain.toml found - using 1.83.0"
    rustup override set 1.83.0
fi

# Install components
echo ""
echo "Installing Rust components..."
rustup component add rustfmt clippy --toolchain 1.83.0
rustup component add rustfmt clippy --toolchain stable
rustup component add miri --toolchain nightly
rustup component add llvm-tools-preview --toolchain nightly || true

# Install cargo tools
echo ""
echo "Installing cargo tools..."

TOOLS=(
    "cargo-fuzz"
    "cargo-audit"
    "cargo-deny"
    "cargo-geiger"
    "cargo-llvm-cov"
    "cargo-msrv"
    "cargo-mutants"
    "cargo-semver-checks"
)

for tool in "${TOOLS[@]}"; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "Installing $tool..."
        cargo install "$tool" --force 2>&1 | tee "$ARTIFACT_DIR/${tool}-install.log" || {
            echo "⚠️ $tool installation failed (non-blocking)"
        }
    else
        echo "✅ $tool already installed"
    fi
done

# Install external tools
echo ""
echo "Installing external tools..."

# Syft
if ! command -v syft >/dev/null 2>&1; then
    echo "Installing Syft..."
    curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin || {
        echo "⚠️ Syft installation failed"
    }
fi

# Grype
if ! command -v grype >/dev/null 2>&1; then
    echo "Installing Grype..."
    curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh -s -- -b /usr/local/bin || {
        echo "⚠️ Grype installation failed"
    }
fi

# cosign
if ! command -v cosign >/dev/null 2>&1; then
    echo "Installing cosign..."
    wget -qO- https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64 -o /dev/null || {
        echo "⚠️ cosign installation failed"
    }
fi

# diffoscope
if ! command -v diffoscope >/dev/null 2>&1; then
    echo "Installing diffoscope..."
    pip install diffoscope || {
        echo "⚠️ diffoscope installation failed"
    }
fi

# Generate tools lock file
echo ""
echo "Generating tools lock file..."
cat > .tools-lock.json << EOF
{
  "rust_toolchain": "$(rustc --version)",
  "cargo_version": "$(cargo --version)",
  "tools": {
    "cargo-fuzz": "$(cargo fuzz --version 2>/dev/null || echo 'not installed')",
    "cargo-audit": "$(cargo audit --version 2>/dev/null || echo 'not installed')",
    "cargo-deny": "$(cargo deny --version 2>/dev/null || echo 'not installed')",
    "cargo-geiger": "$(cargo geiger --version 2>/dev/null || echo 'not installed')",
    "cargo-llvm-cov": "$(cargo llvm-cov --version 2>/dev/null || echo 'not installed')",
    "cargo-msrv": "$(cargo msrv --version 2>/dev/null || echo 'not installed')",
    "cargo-mutants": "$(cargo mutants --version 2>/dev/null || echo 'not installed')",
    "cargo-semver-checks": "$(cargo semver-checks --version 2>/dev/null || echo 'not installed')",
    "syft": "$(syft version 2>/dev/null || echo 'not installed')",
    "grype": "$(grype version 2>/dev/null || echo 'not installed')",
    "miri": "$(rustup +nightly run miri --version 2>/dev/null || echo 'not installed')"
  },
  "date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "✅ Tools lock file generated: .tools-lock.json"

# Verify git signing
echo ""
echo "Verifying git signing..."
if git config --get user.signingkey >/dev/null 2>&1; then
    echo "✅ Git signing configured"
else
    echo "⚠️ Git signing not configured"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Bootstrap complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

