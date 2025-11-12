#!/bin/bash
# CI Health Check - Run all CI checks locally before pushing
set -e

echo "🔍 Running CI Health Check..."
echo "=============================="
echo ""

echo "1. Format check..."
cargo fmt --all -- --check || {
    echo "❌ Format check failed. Run 'cargo fmt --all' to fix."
    exit 1
}
echo "✅ Format: OK"
echo ""

echo "2. Clippy check..."
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "❌ Clippy check failed. Fix warnings and try again."
    exit 1
}
echo "✅ Clippy: OK"
echo ""

echo "3. Build check..."
cargo build --release || {
    echo "❌ Build failed. Fix compilation errors and try again."
    exit 1
}
echo "✅ Build: OK"
echo ""

echo "4. Test check..."
cargo test --lib --all --no-fail-fast || {
    echo "❌ Tests failed. Fix failing tests and try again."
    exit 1
}
echo "✅ Tests: OK"
echo ""

echo ""
echo "✅ All CI checks passed! Safe to push."
