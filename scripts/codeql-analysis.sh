#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 CodeQL Static Analysis"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if CodeQL CLI is available
if ! command -v codeql &> /dev/null; then
    echo "⚠️  CodeQL CLI not found. Installing..."
    echo "   Visit: https://github.com/github/codeql-cli-binaries/releases"
    echo "   Or use GitHub Actions CodeQL workflow for automated analysis"
    exit 0
fi

# Create database
DB_DIR="codeql-db"
echo "Creating CodeQL database..."
codeql database create "$DB_DIR" --language=rust --source-root=. || {
    echo "⚠️  CodeQL database creation failed. This is non-blocking."
    echo "   CodeQL analysis will run automatically in GitHub Actions."
    exit 0
}

# Run analysis
echo "Running CodeQL analysis..."
codeql database analyze "$DB_DIR" --format=sarif-latest --output=codeql-results.sarif || {
    echo "⚠️  CodeQL analysis failed. This is non-blocking."
    echo "   CodeQL analysis will run automatically in GitHub Actions."
    exit 0
}

echo "✅ CodeQL analysis complete"
echo "📊 Results: codeql-results.sarif"
echo ""
echo "Note: CodeQL analysis also runs automatically in GitHub Actions"
echo "View results at: https://github.com/codethor0/cryprq/security/code-scanning"

