# Folder Structure Guide

## Overview

This document describes the organization and structure of the CrypRQ codebase.

## Root Directory

```
cryprq/
 Cargo.toml              # Rust workspace configuration
 Cargo.lock              # Dependency lock file
 Dockerfile              # Main Dockerfile
 docker-compose.yml      # Docker Compose configuration
 .dockerignore           # Docker ignore patterns
 rust-toolchain.toml     # Rust toolchain version
 README.md               # Main README
 LICENSE                 # License file
 SECURITY.md             # Security policy
 CONTRIBUTING.md         # Contribution guidelines

 cli/                    # CLI application
    Cargo.toml
    src/
        main.rs

 crypto/                 # Cryptographic primitives
    Cargo.toml
    src/
        lib.rs
        hybrid.rs
        ppk.rs
        pqc_suite.rs
        zkp.rs

 p2p/                    # Peer-to-peer networking
    Cargo.toml
    src/
        lib.rs

 node/                   # VPN node implementation
    Cargo.toml
    src/
        lib.rs
        padding.rs
        dns.rs
        traffic_shaping.rs
        tls.rs

 core/                   # Core functionality
    Cargo.toml
    src/

 tests/                  # Integration tests (if exists)
    integration_test.rs
    e2e_test.rs

 scripts/                # Build and utility scripts
    test-unit.sh
    test-integration.sh
    test-e2e.sh
    security-audit.sh
    performance-tests.sh
    compliance-checks.sh
    build-android.sh
    test-android.sh
    build-ios.sh
    test-ios.sh
    sync-environments.sh
    cleanup.sh

 docs/                   # Documentation
    DOCKER.md
    TESTING.md
    FOLDER_STRUCTURE.md

 gui/                    # Desktop GUI application
    package.json
    docker-compose.yml
    src/
    electron/

 mobile/                 # Mobile application
    package.json
    docker-compose.yml
    android/
    ios/
    src/

 .github/                # GitHub workflows
    workflows/
        docker-test.yml
        mobile-android.yml
        mobile-ios.yml

 target/                 # Rust build artifacts (gitignored)
```

## Key Directories

### `/cli`
Command-line interface for CrypRQ. Entry point for user interactions.

### `/crypto`
Cryptographic primitives including:
- Hybrid handshake (ML-KEM + X25519)
- Post-Quantum Pre-Shared Keys (PPKs)
- Post-Quantum Cryptography suite
- Zero-Knowledge Proofs

### `/p2p`
Peer-to-peer networking layer handling:
- Peer discovery
- Connection management
- Key rotation

### `/node`
VPN node implementation with:
- Tunnel management
- Packet encryption/decryption
- Traffic shaping
- DNS resolution
- TLS support

### `/scripts`
Utility scripts for:
- Testing (unit, integration, E2E)
- Security auditing
- Performance testing
- Compliance checking
- Mobile builds
- Environment synchronization
- Cleanup

### `/docs`
Documentation including:
- Docker setup guide
- Testing guide
- Folder structure (this file)
- User guides
- API documentation

### `/gui`
Desktop GUI application (Electron/React).

### `/mobile`
Mobile application (React Native) with:
- Android native code
- iOS native code
- Shared TypeScript code

## Build Artifacts (Gitignored)

- `target/` - Rust build artifacts
- `dist/` - Distribution packages
- `node_modules/` - Node.js dependencies
- `*.log` - Log files
- `coverage/` - Test coverage reports

## Best Practices

1. **Keep it organized**: Place files in appropriate directories
2. **Document changes**: Update this file when adding new directories
3. **Clean regularly**: Use `scripts/cleanup.sh` to remove artifacts
4. **Sync environments**: Use `scripts/sync-environments.sh` for remote sync
5. **Follow conventions**: Use consistent naming and structure

## Adding New Components

When adding new components:

1. Create appropriate directory structure
2. Add to workspace `Cargo.toml` if Rust crate
3. Update this documentation
4. Add tests in appropriate test directory
5. Update CI/CD workflows if needed

## References

- [Rust Workspace Documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [CrypRQ README](../README.md)

