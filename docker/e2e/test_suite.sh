#!/bin/bash
# Copyright (c) 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# E2E Test Suite Runner
# Runs comprehensive end-to-end tests for CrypRQ
#
# This suite requires the Docker CLI and daemon to inspect the peer containers.
# Missing prerequisites are errors; process health alone does not prove a handshake.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTS_DIR="${TESTS_DIR:-${SCRIPT_DIR}/tests}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
LISTENER_HOST="${LISTENER_HOST:-listener}"
DIALER_HOST="${DIALER_HOST:-dialer}"
LISTENER_PORT="${LISTENER_PORT:-9999}"
TEST_TIMEOUT="${TEST_TIMEOUT:-60}"

# Test results
PASSED=0
FAILED=0
SKIPPED=0

log_info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

# Wait for services to be ready
wait_for_service() {
    local host=$1
    local port=$2
    local max_attempts=30
    local attempt=0
    local container_logs

    # Container name format: cryprq-e2e-<service>
    local container_name="cryprq-e2e-${host}"

    log_info "Waiting for $host:$port (container: $container_name) to be ready..."

    if ! command -v docker >/dev/null 2>&1; then
        log_error "UNRESOLVED: Docker CLI is required for peer verification; no protocol checks ran."
        return 1
    fi
    if ! docker info >/dev/null 2>&1; then
        log_error "UNRESOLVED: Docker daemon is unavailable; no protocol checks ran."
        return 1
    fi

    # Use docker logs when docker *is* available (local runs, host scripts)
    while [ $attempt -lt $max_attempts ]; do
        # First, verify container exists and is running
        if ! docker ps --format '{{.Names}}' | grep -q "^${container_name}\$"; then
            attempt=$((attempt + 1))
            sleep 1
            continue
        fi

        container_logs=$(docker logs "$container_name" 2>&1)
        # For listener: check for exact event=listener_ready log line
        if [ "$host" = "listener" ]; then
            # Match the exact log format: event=listener_ready ... status=accepting_connections
            if grep -q 'event=listener_ready.*status=accepting_connections' <<< "$container_logs"; then
                log_info "$host:$port is ready (listener_ready event seen in logs)"
                return 0
            fi
        else
            # For dialer: check for connection_established or handshake_complete
            if grep -qE 'event=(connection_established|handshake_complete)' <<< "$container_logs"; then
                log_info "$host:$port is ready (connection established)"
                return 0
            fi
        fi

        attempt=$((attempt + 1))
        sleep 1
    done

    log_error "$host:$port failed to become ready after $max_attempts attempts"

    # Guard log dump as well - only if docker is available
    if command -v docker >/dev/null 2>&1; then
        log_error "Container $container_name logs (last 20 lines):"
        docker logs "$container_name" 2>&1 | tail -20 || true
    else
        log_warn "docker CLI not available; cannot dump container logs."
    fi

    return 1
}

# The tests this harness must run, all of them, exactly once.
REQUIRED_TESTS="01_legacy_quic_connection.sh 02_legacy_rotation_timer.sh"
REQUIRED_COUNT=2

# Run a test script
run_test() {
    local test_file=$1
    local test_name=$(basename "$test_file" .sh)

    log_info "Running test: $test_name"
    
    if [ ! -f "$test_file" ]; then
        log_error "Test file not found: $test_file"
        FAILED=$((FAILED + 1))
        return 1
    fi

    if [ ! -x "$test_file" ]; then
        chmod +x "$test_file"
    fi

    if timeout "$TEST_TIMEOUT" bash "$test_file"; then
        log_info "[PASS] Test passed: $test_name"
        PASSED=$((PASSED + 1))
        return 0
    else
        log_error "[FAIL] Test failed: $test_name"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# Main test execution
main() {
    log_info "=== CrypRQ legacy transport E2E (libp2p/QUIC test mode) ==="
    log_info "Listener: $LISTENER_HOST:$LISTENER_PORT"
    log_info "Dialer: $DIALER_HOST"
    echo ""

    # An absent/partial test directory must never produce a zero-test success.
    local required
    # This harness covers the legacy transport only: that two test-mode
    # containers connect over QUIC and that the local rotation timer runs.
    # It asserts nothing about the hybrid handshake, file integrity or record
    # validation. Those are asserted by the authenticated E2E scripts and the
    # Rust tests listed in docs/E2E_COVERAGE_MAP.md, which the same workflow
    # runs as its own job.
    for required in $REQUIRED_TESTS; do
        if [ ! -f "$TESTS_DIR/$required" ]; then
            log_error "UNRESOLVED: Required test is missing: $TESTS_DIR/$required"
            exit 1
        fi
    done

    # Wait for services
    # Note: wait_for_service now uses process/log checks instead of nc -z for UDP
    wait_for_service "$LISTENER_HOST" "$LISTENER_PORT" || exit 1
    wait_for_service "$DIALER_HOST" "$LISTENER_PORT" || exit 1

    # Run all tests
    log_info "Starting test execution..."
    echo ""

    for required in $REQUIRED_TESTS; do
        run_test "$TESTS_DIR/$required" || true
        echo ""
    done

    # Print summary
    echo ""
    log_info "=== Test Summary ==="
    log_info "Passed: $PASSED"
    log_error "Failed: $FAILED"
    log_warn "Skipped: $SKIPPED"
    printf 'E2E_RESULT passed=%s failed=%s skipped=%s\n' "$PASSED" "$FAILED" "$SKIPPED"
    echo ""

    if [ "$FAILED" -eq 0 ] && [ "$SKIPPED" -eq 0 ] && [ "$PASSED" -eq "$REQUIRED_COUNT" ]; then
        log_info "All legacy transport tests passed. This is not a result for the authenticated protocol."
        exit 0
    else
        log_error "Some tests failed"
        exit 1
    fi
}

main "$@"

