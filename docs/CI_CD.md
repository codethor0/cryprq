# CI/CD Guide

## Overview

This guide covers the Continuous Integration and Continuous Deployment (CI/CD) pipelines for CrypRQ, including GitHub Actions workflows and local testing.

## GitHub Actions Workflows

### Available Workflows

1. **CI** (`.github/workflows/ci.yml`)
   - Runs on push and pull requests
   - Builds and tests the application
   - Checks code formatting and linting

2. **Docker Tests** (`.github/workflows/docker-test.yml`)
   - Builds Docker images
   - Tests Docker Compose setup
   - Runs integration tests

3. **Security Audit** (`.github/workflows/security-audit.yml`)
   - Runs cargo-audit
   - Checks for known vulnerabilities
   - Runs cargo-deny

4. **CodeQL** (`.github/workflows/codeql.yml`)
   - Static code analysis
   - Security vulnerability scanning
   - Runs on schedule and PRs

5. **Mobile CI** (`.github/workflows/mobile-android.yml`, `.github/workflows/mobile-ios.yml`)
   - Builds Android APKs
   - Builds iOS archives
   - Runs mobile tests

### Workflow Triggers

- **Push to main**: All workflows run
- **Pull requests**: CI and security checks run
- **Manual dispatch**: Can trigger workflows manually
- **Scheduled**: Security audits run weekly

## Local CI Simulation

### Run All Checks

```bash
# Run production readiness script
bash scripts/finalize-production.sh
```

### Individual Checks

```bash
# Compilation
cargo check --workspace

# Build
cargo build --release

# Tests
cargo test --all

# Formatting
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Security
bash scripts/security-audit.sh

# Compliance
bash scripts/compliance-checks.sh
```

## CI Pipeline Steps

### 1. Checkout

```yaml
- uses: actions/checkout@v4
```

### 2. Setup Rust

```yaml
- uses: dtolnay/rust-toolchain@master
  with:
    toolchain: 1.83.0
    components: rustfmt, clippy
```

### 3. Cache Dependencies

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

### 4. Build and Test

```yaml
- name: Build
  run: cargo build --release

- name: Test
  run: cargo test --all
```

## Docker CI Pipeline

### Build Docker Image

```yaml
- name: Build Docker image
  run: docker build -t cryprq-node:test -f Dockerfile .
```

### Test Docker Setup

```yaml
- name: Test Docker Compose
  run: |
    docker compose up -d cryprq-listener
    sleep 5
    docker compose run --rm cryprq-dialer
    docker compose down -v
```

## Mobile CI Pipeline

### Android

```yaml
- name: Set up JDK
  uses: actions/setup-java@v4
  with:
    distribution: 'temurin'
    java-version: '17'

- name: Set up Android SDK
  uses: android-actions/setup-android@v3

- name: Build Android APK
  working-directory: mobile/android
  run: |
    chmod +x gradlew
    ./gradlew assembleDebug
```

### iOS

```yaml
- name: Set up Node.js
  uses: actions/setup-node@v4
  with:
    node-version: '18'

- name: Install CocoaPods
  run: gem install cocoapods

- name: Build iOS
  working-directory: mobile/ios
  run: |
    pod install
    xcodebuild -workspace CrypRQ.xcworkspace \
      -scheme CrypRQ \
      -configuration Debug \
      -sdk iphonesimulator \
      build
```

## Artifacts

### Build Artifacts

- Release binaries (Linux, macOS, Windows)
- Docker images
- Mobile APKs/IPAs

### Test Reports

- Test results (JUnit format)
- Coverage reports
- Security audit reports

## Deployment Pipeline

### Release Process

1. **Create Release Branch**:
```bash
git checkout -b release/v1.0.0
```

2. **Update Version**:
```bash
# Update Cargo.toml versions
# Update CHANGELOG.md
```

3. **Run Tests**:
```bash
bash scripts/finalize-production.sh
```

4. **Create Tag**:
```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

5. **GitHub Release**:
   - Create release on GitHub
   - Attach artifacts
   - Publish release notes

### Automated Deployment

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build release
        run: cargo build --release
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: release-binaries
          path: target/release/cryprq
```

## Monitoring CI/CD

### GitHub Actions Status

- View workflow runs: `https://github.com/codethor0/cryprq/actions`
- Check specific workflow: Click on workflow name
- View logs: Click on failed job

### Local Testing

```bash
# Test CI locally
act -l  # List workflows
act -j build  # Run build job
```

## Troubleshooting

### Failed Builds

1. **Check logs**: Review GitHub Actions logs
2. **Reproduce locally**: Run same commands locally
3. **Check dependencies**: Verify Cargo.lock is up to date

### Test Failures

1. **Flaky tests**: Add retries or fix race conditions
2. **Environment differences**: Check OS/version differences
3. **Dependencies**: Verify all dependencies are available

### Docker Issues

1. **Build failures**: Check Dockerfile syntax
2. **Network issues**: Verify Docker network configuration
3. **Resource limits**: Check available disk space/memory

## Best Practices

1. **Keep workflows fast**: Cache dependencies, parallelize jobs
2. **Test before merge**: Require PR checks to pass
3. **Automate releases**: Use tags to trigger releases
4. **Monitor failures**: Set up notifications for failed builds
5. **Document changes**: Update workflows when making changes

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI Best Practices](https://github.com/rust-lang/cargo/blob/master/CONTRIBUTING.md)
- [Docker CI/CD Guide](DOCKER.md)

