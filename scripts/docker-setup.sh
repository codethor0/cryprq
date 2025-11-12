#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🐳 Docker Environment Setup"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker:"
    echo "   macOS: https://docs.docker.com/desktop/install/mac-install/"
    echo "   Linux: https://docs.docker.com/engine/install/"
    echo "   Windows: https://docs.docker.com/desktop/install/windows-install/"
    exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo "❌ Docker Compose not found. Please install Docker Compose."
    exit 1
fi

echo "✅ Docker version: $(docker --version)"
echo "✅ Docker Compose version: $(docker compose version 2>/dev/null || docker-compose --version)"
echo ""

# Check Docker daemon
if ! docker info &> /dev/null; then
    echo "❌ Docker daemon not running. Please start Docker."
    exit 1
fi

echo "✅ Docker daemon is running"
echo ""

# Build images
echo "Building Docker images..."
docker compose build || {
    echo "❌ Docker build failed"
    exit 1
}

echo "✅ Docker images built successfully"
echo ""

# Verify images
echo "Verifying Docker images..."
docker images | grep -E "cryprq|REPOSITORY" || echo "⚠️  No CrypRQ images found"
echo ""

echo "✅ Docker environment setup complete!"
echo ""
echo "Next steps:"
echo "  1. Start containers: docker compose up -d"
echo "  2. Run QA tests: bash scripts/docker-qa-suite.sh"
echo "  3. Check status: docker compose ps"

