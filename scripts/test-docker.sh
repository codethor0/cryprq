#!/usr/bin/env bash

# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT

# Standardized Docker test runner for CrypRQ
# Usage:
#   ./scripts/test-docker.sh              # Run unit tests
#   ./scripts/test-docker.sh integration  # Run integration tests
#   ./scripts/test-docker.sh all          # Run all tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
COMPOSE_FILE="$PROJECT_ROOT/docker-compose.test.yml"

cd "$PROJECT_ROOT"

log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" >&2
}

case "${1:-unit}" in
    unit)
        log "Running unit tests in Docker..."
        docker compose -f "$COMPOSE_FILE" --profile test run --rm test
        ;;
    integration)
        log "Running integration tests in Docker..."
        log "Note: This requires the listener service to be running"
        docker compose -f "$COMPOSE_FILE" --profile test-integration up --abort-on-container-exit test-integration
        docker compose -f "$COMPOSE_FILE" --profile test-integration down
        ;;
    all)
        log "Running all tests..."
        log "Step 1: Unit tests"
        docker compose -f "$COMPOSE_FILE" --profile test run --rm test
        log "Step 2: Integration tests"
        docker compose -f "$COMPOSE_FILE" --profile test-integration up --abort-on-container-exit test-integration
        docker compose -f "$COMPOSE_FILE" --profile test-integration down
        ;;
    build)
        log "Building test Docker image..."
        docker build -f "$PROJECT_ROOT/Dockerfile.test" -t cryprq-test:latest "$PROJECT_ROOT"
        ;;
    *)
        echo "Usage: $0 [unit|integration|all|build]" >&2
        echo "" >&2
        echo "Commands:" >&2
        echo "  unit         Run unit tests (default)" >&2
        echo "  integration  Run integration tests (requires listener)" >&2
        echo "  all          Run unit + integration tests" >&2
        echo "  build        Build test Docker image" >&2
        exit 1
        ;;
esac

log "Tests completed successfully!"

