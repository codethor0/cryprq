# Docker Setup and Testing Guide

## Overview

CrypRQ provides comprehensive Docker support for building, testing, and deploying the application in containerized environments.

## Prerequisites

- Docker Engine 20.10+ or Docker Desktop
- Docker Compose v2.0+ (or docker-compose v1.29+)
- 2GB+ free disk space
- 1GB+ RAM available

## Quick Start

### Build Docker Image

```bash
docker build -t cryprq-node:latest -f Dockerfile .
```

### Run Container

```bash
# Run as listener
docker run -d --name cryprq-listener -p 9999:9999/udp cryprq-node:latest \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1

# Run as dialer
docker run --rm cryprq-node:latest \
  --peer /ip4/<LISTENER_IP>/udp/9999/quic-v1
```

## Docker Compose

### Basic Setup

```bash
# Start listener
docker compose up -d cryprq-listener

# Run dialer test
docker compose run --rm cryprq-dialer

# Stop and clean up
docker compose down -v
```

### Service Configuration

The `docker-compose.yml` file defines three services:

1. **cryprq-listener**: Main listener node (always running)
2. **cryprq-dialer**: Test dialer node (profile: test)
3. **test-runner**: Cargo test runner (profile: test)

### Environment Variables

- `RUST_LOG`: Log level (default: `info`)
- `PORT`: UDP port (default: `9999`)

## Testing

### Unit Tests

```bash
bash scripts/test-unit.sh
```

### Integration Tests

```bash
bash scripts/test-integration.sh
```

### End-to-End Tests

```bash
bash scripts/test-e2e.sh
```

### All Tests

```bash
docker compose --profile test up --abort-on-container-exit
```

## Security

### Security Audit

```bash
bash scripts/security-audit.sh
```

This runs:
- `cargo-audit` for known vulnerabilities
- `cargo-deny` for dependency checks
- Unsafe code detection
- Hardcoded secret detection

### Best Practices

1. **Use multi-stage builds** (already implemented)
2. **Minimize image size** (using slim base images)
3. **Run as non-root** (configured in Dockerfile)
4. **Keep dependencies updated** (run `cargo update` regularly)
5. **Scan images** with security scanners

## Performance

### Performance Tests

```bash
bash scripts/performance-tests.sh
```

Measures:
- Connection handshake time
- Memory usage
- Binary size

### Optimization Tips

1. Use `--release` builds for production
2. Enable Docker BuildKit for faster builds
3. Use layer caching effectively
4. Minimize COPY operations

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs cryprq-listener

# Check container status
docker ps -a

# Inspect container
docker inspect cryprq-listener
```

### Network Issues

```bash
# Check network
docker network ls
docker network inspect cryprq-network

# Test connectivity
docker exec cryprq-listener ping -c 3 8.8.8.8
```

### Build Failures

```bash
# Clean build cache
docker builder prune

# Rebuild without cache
docker build --no-cache -t cryprq-node:latest -f Dockerfile .
```

## CI/CD Integration

### GitHub Actions

The repository includes GitHub Actions workflows:

- `.github/workflows/docker-test.yml`: Docker build and test
- `.github/workflows/mobile-android.yml`: Android CI
- `.github/workflows/mobile-ios.yml`: iOS CI

### Local CI Simulation

```bash
# Run all checks locally
bash scripts/test-unit.sh
bash scripts/test-integration.sh
bash scripts/test-e2e.sh
bash scripts/security-audit.sh
bash scripts/compliance-checks.sh
```

## Advanced Usage

### Custom Networks

```bash
docker network create cryprq-custom
docker run --network cryprq-custom cryprq-node:latest
```

### Volume Mounts

```bash
docker run -v $(pwd)/config:/app/config cryprq-node:latest
```

### Resource Limits

```bash
docker run --memory="512m" --cpus="1.0" cryprq-node:latest
```

## References

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [CrypRQ README](../README.md)

