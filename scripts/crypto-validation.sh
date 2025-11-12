#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔐 Cryptographic Algorithm Validation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run crypto tests
echo "Running cryptographic algorithm tests..."
cargo test --lib -p cryprq-crypto --no-fail-fast || {
    echo "❌ Cryptographic tests failed"
    exit 1
}

# Run hybrid handshake tests
echo "Running hybrid handshake tests..."
cargo test --lib -p cryprq-crypto hybrid --no-fail-fast || {
    echo "❌ Hybrid handshake tests failed"
    exit 1
}

# Run PPK tests
echo "Running PPK (Post-Quantum Pre-Shared Key) tests..."
cargo test --lib -p cryprq-crypto ppk --no-fail-fast || {
    echo "❌ PPK tests failed"
    exit 1
}

# Run PQC suite tests
echo "Running PQC suite tests..."
cargo test --lib -p cryprq-crypto pqc --no-fail-fast || {
    echo "❌ PQC suite tests failed"
    exit 1
}

# Run ZKP tests
echo "Running Zero-Knowledge Proof tests..."
cargo test --lib -p cryprq-crypto zkp --no-fail-fast || {
    echo "❌ ZKP tests failed"
    exit 1
}

# Run p2p tests (includes crypto operations)
echo "Running p2p tests (includes crypto operations)..."
cargo test --lib -p p2p --no-fail-fast || {
    echo "❌ p2p tests failed"
    exit 1
}

echo ""
echo "✅ All cryptographic algorithm tests passed"
echo ""
echo "Validated algorithms:"
echo "  ✅ ML-KEM (Kyber768) + X25519 hybrid handshake"
echo "  ✅ Post-Quantum Pre-Shared Keys (PPKs)"
echo "  ✅ PQC Suite (ML-KEM768, ML-KEM1024, X25519)"
echo "  ✅ Zero-Knowledge Proofs framework"
echo "  ✅ Key rotation and expiration"

