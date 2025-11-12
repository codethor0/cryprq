# Docker Testing Guide

This guide describes how to run extensive quality assurance tests using Docker for CrypRQ.

## Prerequisites

- Docker installed and running
- Docker Compose installed
- Sufficient disk space (at least 5GB)

## Quick Start

### 1. Setup Docker Environment

```bash
bash scripts/docker-setup.sh
```

This will:
- Check Docker installation
- Verify Docker daemon is running
- Build Docker images
- Verify images are created

### 2. Run Complete QA Suite

```bash
bash scripts/docker-qa-suite.sh
```

This comprehensive test suite runs:
- Secret scanning
- Dependency scanning
- Static analysis
- Dynamic analysis
- Security audits
- Cryptographic validation
- Unit tests
- Integration tests
- End-to-end tests
- Performance tests
- Build tests
- Code quality checks

### 3. Run Individual Tests

```bash
# Run specific test
bash scripts/docker-test-individual.sh <test-name>

# Available test names:
# - secret-scan
# - dependency-scan
# - static-analysis
# - dynamic-analysis
# - security-audit
# - crypto-validation
# - unit-tests
# - integration-tests
# - e2e-tests
# - performance-tests
# - build
# - code-quality
# - all (runs complete suite)
```

## Manual Docker Commands

### Start Containers

```bash
# Start all services
docker compose up -d

# Start specific service
docker compose up -d cryprq-listener
```

### Check Container Status

```bash
# List running containers
docker compose ps

# View logs
docker compose logs cryprq-listener
docker compose logs cryprq-dialer
```

### Run Tests in Container

```bash
# Create test runner container
docker compose run -d --name cryprq-test-runner cryprq-test-runner sleep 3600

# Run specific test
docker exec -it cryprq-test-runner bash -c "bash scripts/secret-scan.sh"
docker exec -it cryprq-test-runner bash -c "bash scripts/crypto-validation.sh"
docker exec -it cryprq-test-runner bash -c "cargo test --lib --all"
```

### Interactive Shell

```bash
# Get interactive shell in container
docker exec -it cryprq-test-runner bash

# Inside container, you can run:
cargo test
cargo build --release
bash scripts/crypto-validation.sh
```

## Test Categories

### 1. Secret Scanning

Detects hardcoded secrets and sensitive information.

```bash
docker exec -it cryprq-test-runner bash -c "bash scripts/secret-scan.sh"
```

### 2. Dependency Scanning

Scans dependencies for vulnerabilities.

```bash
docker exec -it cryprq-test-runner bash -c "cargo audit --deny warnings"
docker exec -it cryprq-test-runner bash -c "cargo deny check"
```

### 3. Static Analysis

CodeQL static security analysis.

```bash
docker exec -it cryprq-test-runner bash -c "bash scripts/codeql-analysis.sh"
```

### 4. Dynamic Analysis

Runtime testing and memory leak detection.

```bash
docker exec -it cryprq-test-runner bash -c "bash scripts/dynamic-analysis.sh"
```

### 5. Security Audits

Comprehensive security validation.

```bash
docker exec -it cryprq-test-runner bash -c "bash scripts/security-audit.sh"
```

### 6. Cryptographic Validation

Validates all cryptographic algorithms.

```bash
docker exec -it cryprq-test-runner bash -c "bash scripts/crypto-validation.sh"
```

### 7. Unit Tests

Runs all unit tests.

```bash
docker exec -it cryprq-test-runner bash -c "cargo test --lib --all --no-fail-fast"
```

### 8. Integration Tests

Runs integration tests.

```bash
docker exec -it cryprq-test-runner bash -c "bash scripts/test-integration.sh"
```

### 9. End-to-End Tests

Full application flow testing.

```bash
docker exec -it cryprq-test-runner bash -c "bash scripts/end-to-end-tests.sh"
```

### 10. Performance Tests

Performance benchmarks.

```bash
docker exec -it cryprq-test-runner bash -c "bash scripts/performance-tests.sh"
```

### 11. Build Tests

Release build verification.

```bash
docker exec -it cryprq-test-runner bash -c "cargo build --release -p cryprq"
```

### 12. Code Quality

Formatting and linting checks.

```bash
docker exec -it cryprq-test-runner bash -c "cargo fmt --all -- --check"
docker exec -it cryprq-test-runner bash -c "cargo clippy --all-targets --all-features -- -D warnings"
```

## Docker Compose Services

### cryprq-listener

Listens for incoming connections.

```bash
docker compose up -d cryprq-listener
docker compose logs cryprq-listener
```

### cryprq-dialer

Connects to listener (runs once and exits).

```bash
docker compose run --rm cryprq-dialer
```

### cryprq-test-runner

Test runner container (keeps running for interactive testing).

```bash
docker compose run -d --name cryprq-test-runner cryprq-test-runner sleep 3600
docker exec -it cryprq-test-runner bash
```

## Troubleshooting

### Container Won't Start

```bash
# Check Docker daemon
docker info

# Check logs
docker compose logs

# Rebuild images
docker compose build --no-cache
```

### Tests Fail in Container

```bash
# Check container status
docker compose ps

# View container logs
docker logs cryprq-test-runner

# Get interactive shell to debug
docker exec -it cryprq-test-runner bash
```

### Out of Disk Space

```bash
# Clean up Docker resources
docker system prune -a

# Remove unused images
docker image prune -a
```

### Port Conflicts

If port 9999 is already in use:

```yaml
# Edit docker-compose.yml to use different port
ports:
  - "9998:9999/udp"
```

## Test Results

Test results are logged to:
- `docker-qa-YYYYMMDD-HHMMSS.log` - Complete QA suite log
- Individual test scripts may create their own log files

## CI Integration

All Docker tests are integrated into CI workflows:
- `.github/workflows/ci.yml` - Basic Docker tests
- `.github/workflows/docker-test.yml` - Docker-specific tests
- `.github/workflows/security-checks.yml` - Security tests in Docker

## Best Practices

1. **Run locally first**: Always run Docker tests locally before pushing
2. **Clean up**: Remove containers and images when done
3. **Check logs**: Review test logs for failures
4. **Update images**: Rebuild images when dependencies change
5. **Monitor resources**: Check Docker resource usage

## Summary

Docker testing provides:
- ✅ Isolated test environment
- ✅ Consistent test results
- ✅ Easy cleanup
- ✅ Reproducible tests
- ✅ CI/CD integration

For questions or issues, see:
- `docs/SECURITY_CHECKS.md` - Security checks guide
- `docs/TESTING.md` - General testing guide
- `CONTRIBUTING.md` - Contribution guidelines

