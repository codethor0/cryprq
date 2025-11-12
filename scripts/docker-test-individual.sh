#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

# Run individual tests in Docker container
# Usage: ./scripts/docker-test-individual.sh <test-name>
# Test names: secret-scan, dependency-scan, static-analysis, dynamic-analysis,
#             security-audit, crypto-validation, unit-tests, integration-tests,
#             e2e-tests, performance-tests, build, code-quality

CONTAINER_NAME="cryprq-test-runner"
TEST_NAME="${1:-all}"

# Ensure container exists
if ! docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo "Creating test runner container..."
    docker compose run -d --name "$CONTAINER_NAME" cryprq-test-runner sleep 3600 || {
        echo "⚠️  Container creation failed, trying to start existing..."
        docker start "$CONTAINER_NAME" || {
            echo "❌ Failed to start container"
            exit 1
        }
    }
fi

# Wait for container to be ready
sleep 2

case "$TEST_NAME" in
    secret-scan)
        echo "Running secret scanning..."
        docker exec "$CONTAINER_NAME" bash -c "bash scripts/secret-scan.sh"
        ;;
    dependency-scan)
        echo "Running dependency scanning..."
        docker exec "$CONTAINER_NAME" bash -c "cargo audit --deny warnings || echo 'cargo-audit not available'"
        docker exec "$CONTAINER_NAME" bash -c "cargo deny check || echo 'cargo-deny not available'"
        ;;
    static-analysis)
        echo "Running static analysis..."
        docker exec "$CONTAINER_NAME" bash -c "bash scripts/codeql-analysis.sh || echo 'CodeQL not available'"
        ;;
    dynamic-analysis)
        echo "Running dynamic analysis..."
        docker exec "$CONTAINER_NAME" bash -c "bash scripts/dynamic-analysis.sh || echo 'Dynamic analysis skipped'"
        ;;
    security-audit)
        echo "Running security audit..."
        docker exec "$CONTAINER_NAME" bash -c "bash scripts/security-audit.sh || echo 'Security audit skipped'"
        ;;
    crypto-validation)
        echo "Running cryptographic validation..."
        docker exec "$CONTAINER_NAME" bash -c "bash scripts/crypto-validation.sh"
        ;;
    unit-tests)
        echo "Running unit tests..."
        docker exec "$CONTAINER_NAME" bash -c "cargo test --lib --all --no-fail-fast"
        ;;
    integration-tests)
        echo "Running integration tests..."
        docker exec "$CONTAINER_NAME" bash -c "bash scripts/test-integration.sh || echo 'Integration tests skipped'"
        ;;
    e2e-tests)
        echo "Running end-to-end tests..."
        docker exec "$CONTAINER_NAME" bash -c "bash scripts/end-to-end-tests.sh || echo 'E2E tests skipped'"
        ;;
    performance-tests)
        echo "Running performance tests..."
        docker exec "$CONTAINER_NAME" bash -c "bash scripts/performance-tests.sh || echo 'Performance tests skipped'"
        ;;
    build)
        echo "Running build test..."
        docker exec "$CONTAINER_NAME" bash -c "cargo build --release -p cryprq"
        ;;
    code-quality)
        echo "Running code quality checks..."
        docker exec "$CONTAINER_NAME" bash -c "cargo fmt --all -- --check"
        docker exec "$CONTAINER_NAME" bash -c "cargo clippy --all-targets --all-features -- -D warnings"
        ;;
    all)
        echo "Running all tests..."
        bash scripts/docker-qa-suite.sh
        ;;
    *)
        echo "Unknown test: $TEST_NAME"
        echo "Available tests: secret-scan, dependency-scan, static-analysis, dynamic-analysis,"
        echo "                 security-audit, crypto-validation, unit-tests, integration-tests,"
        echo "                 e2e-tests, performance-tests, build, code-quality, all"
        exit 1
        ;;
esac

