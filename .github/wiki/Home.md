# CrypRQ Wiki

Welcome to the CrypRQ Wiki. This wiki contains comprehensive documentation for the CrypRQ post-quantum VPN project.

## Quick Links

- [Getting Started](Getting-Started)
- [Architecture Overview](Architecture)
- [VPN Setup Guide](VPN-Setup)
- [Testing Guide](Testing)
- [Troubleshooting](Troubleshooting)
- [Development Guide](Development)

## Overview

CrypRQ is a post-quantum, zero-trust VPN solution that implements:
- Hybrid ML-KEM (Kyber768) + X25519 handshake over libp2p QUIC
- Five-minute ephemeral key rotation with secure zeroization
- System-wide VPN routing via TUN interface
- Docker-based VPN solution with web UI
- Comprehensive testing and verification

## Key Features

- **Post-Quantum Cryptography**: ML-KEM (Kyber768) + X25519 hybrid handshake
- **Zero-Trust Architecture**: No persistent keys, no PKI, no trusted third parties
- **Key Rotation**: Five-minute ephemeral key rotation with secure zeroization
- **Packet Forwarding**: Full bidirectional packet forwarding over libp2p
- **Docker Support**: Complete containerized VPN solution
- **Web UI**: React-based management interface
- **Comprehensive Testing**: 14 test categories, all passing

## Documentation

All documentation is available in the `/docs` directory of the repository. Key documents include:

- [Comprehensive Test Report](../docs/COMPREHENSIVE_TEST_REPORT.md)
- [Docker VPN Setup](../docs/DOCKER_VPN_SETUP.md)
- [VPN Testing Guide](../docs/VPN_TESTING_GUIDE.md)
- [Build Status](../docs/BUILD_STATUS.md)
- [Technology Verification](../docs/TECHNOLOGY_VERIFICATION.md)

## Support

For issues, questions, or contributions, please see:
- [GitHub Issues](https://github.com/codethor0/cryprq/issues)
- [Security Policy](../SECURITY.md)
- [Contributing Guide](../CONTRIBUTING.md)

