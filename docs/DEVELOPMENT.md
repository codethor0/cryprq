# Development Guide

## Overview

This guide covers setting up a local development environment for CrypRQ, including dependencies, build processes, and development workflows.

## Prerequisites

### Required Tools

- **Rust 1.83.0+**: Install via [rustup](https://rustup.rs/)
- **Docker** (optional): For containerized development
- **Git**: Version control
- **Cargo**: Comes with Rust installation

### Platform-Specific Requirements

#### macOS
- Xcode Command Line Tools: `xcode-select --install`
- Homebrew (optional): For additional tools

#### Linux
- Build essentials: `sudo apt-get install build-essential` (Debian/Ubuntu)
- OpenSSL development libraries: `sudo apt-get install libssl-dev`

#### Windows
- Visual Studio Build Tools or Visual Studio Community
- Windows Subsystem for Linux (WSL) recommended

## Environment Setup

### 1. Install Rust Toolchain

```bash
# Install rustup if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Rust 1.83.0
rustup install 1.83.0
rustup default 1.83.0

# Install required components
rustup component add rustfmt clippy
```

### 2. Clone Repository

```bash
git clone https://github.com/codethor0/cryprq.git
cd cryprq
```

### 3. Verify Installation

```bash
rustc --version  # Should show 1.83.0 or later
cargo --version
```

## Building the Project

### Development Build

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p cryprq-crypto
cargo build -p node
cargo build -p p2p
cargo build -p cli
```

### Release Build

```bash
# Optimized release build
cargo build --release

# Binary location
./target/release/cryprq
```

### Build for Specific Target

```bash
# Linux (musl - static binary)
cargo build --release --target x86_64-unknown-linux-musl

# macOS
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

## Running Tests

### Unit Tests

```bash
# Run all unit tests
cargo test --lib --all

# Run tests for specific crate
cargo test -p cryprq-crypto

# Run with output
cargo test --lib --all -- --nocapture
```

### Integration Tests

```bash
# Run integration tests
bash scripts/test-integration.sh

# Run E2E tests
bash scripts/test-e2e.sh
```

### Test Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

- Write code following Rust best practices
- Add tests for new functionality
- Update documentation

### 3. Run Checks Before Commit

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all

# Run security audit
bash scripts/security-audit.sh

# Run compliance checks
bash scripts/compliance-checks.sh
```

### 4. Commit Changes

```bash
git add .
git commit -m "feat: add new feature"
```

### 5. Push and Create PR

```bash
git push origin feature/my-feature
# Create PR on GitHub
```

## Code Style

### Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

### Linting

```bash
# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix issues
cargo clippy --fix --allow-dirty
```

### Code Standards

- Follow Rust naming conventions
- Use `cargo fmt` for consistent formatting
- Address all clippy warnings
- Write comprehensive tests
- Document public APIs

## Debugging

### Debug Build

```bash
cargo build
# Binary: ./target/debug/cryprq
```

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run --bin cryprq
```

### Using a Debugger

#### VS Code

Install the "CodeLLDB" extension for debugging Rust code.

#### GDB/LLDB

```bash
# Linux (GDB)
gdb ./target/debug/cryprq

# macOS (LLDB)
lldb ./target/debug/cryprq
```

## Docker Development

### Build Docker Image

```bash
docker build -t cryprq-dev -f Dockerfile .
```

### Run in Docker

```bash
# Run listener
docker run --rm -p 9999:9999/udp cryprq-dev --listen /ip4/0.0.0.0/udp/9999/quic-v1

# Run dialer
docker run --rm cryprq-dev --peer /ip4/<LISTENER_IP>/udp/9999/quic-v1
```

### Docker Compose

```bash
# Start services
docker compose up -d

# View logs
docker compose logs -f

# Stop services
docker compose down
```

## Common Tasks

### Clean Build Artifacts

```bash
cargo clean
```

### Update Dependencies

```bash
cargo update
```

### Check for Outdated Dependencies

```bash
cargo outdated
```

### Generate Documentation

```bash
# Build docs
cargo doc --no-deps --all --open
```

## Troubleshooting

### Build Failures

1. **Missing dependencies**: Install platform-specific dependencies
2. **Rust version**: Ensure Rust 1.83.0+ is installed
3. **Clean build**: Try `cargo clean && cargo build`

### Test Failures

1. **Check logs**: Run with `--nocapture` flag
2. **Docker issues**: Ensure Docker is running
3. **Port conflicts**: Check if ports are already in use

### Common Issues

- **"linker not found"**: Install build tools for your platform
- **"cannot find crate"**: Run `cargo update`
- **"permission denied"**: Check file permissions

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [CrypRQ Testing Guide](TESTING.md)
- [CrypRQ Deployment Guide](DEPLOYMENT.md)

