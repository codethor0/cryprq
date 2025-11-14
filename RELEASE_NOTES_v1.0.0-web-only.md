# CrypRQ v1.0.0-web-only Release Notes

**Release Date:** November 14, 2025

## Overview

CrypRQ v1.0.0-web-only marks the first production-ready release of the web-focused CrypRQ VPN stack with fully functional encrypted file transfer. This release represents a major milestone in post-quantum VPN technology, combining hybrid ML-KEM + X25519 cryptography with a modern web interface and secure file transfer capabilities.

## What's New

### 🎉 File Transfer Over Encrypted Tunnel

**Status: ✅ Fully Functional**

CrypRQ now supports secure file transfer over the encrypted tunnel with end-to-end encryption and integrity verification:

- **CLI Commands**: `send-file` and `receive-file` subcommands
- **Chunked Transfer**: Automatic splitting and reassembly of large files
- **SHA-256 Verification**: Automatic hash verification ensures data integrity
- **Web UI Integration**: File transfer available via web interface
- **Real-time Progress**: Detailed logging for transfer monitoring

**Example Usage:**
```bash
# Terminal 1: Receiver
cryprq receive-file --listen /ip4/0.0.0.0/udp/9999/quic-v1 --output-dir ./received

# Terminal 2: Sender
cryprq send-file --peer /ip4/127.0.0.1/udp/9999/quic-v1/p2p/<PEER_ID> --file sample.txt
```

### 🔧 Critical Bug Fixes

**libp2p Request-Response Protocol Fix:**
- Fixed `PacketCodec` to always length-prefix requests for proper protocol negotiation
- Resolved swarm event loop blocking that prevented protocol negotiation completion
- Ensured swarm lock is released immediately after `send_request()` calls
- Added proper async task yielding to allow event loop processing

**Event Loop Improvements:**
- Fixed premature event loop exit that prevented file transfer completion
- Improved response tracking and timeout handling
- Better error handling for protocol negotiation failures

## Technical Details

### Architecture

- **Core**: Rust workspace with `crypto`, `p2p`, `node`, `cli` crates
- **Web Stack**: React + TypeScript frontend, Node.js Express backend
- **Transport**: libp2p QUIC with hybrid ML-KEM (Kyber768-compatible) + X25519 handshake
- **Encryption**: ChaCha20-Poly1305 AEAD for tunnel data
- **Key Rotation**: Automatic 5-minute ephemeral key rotation with secure zeroization

### Security Features

- **Post-Quantum Cryptography**: ML-KEM mitigates store-now-decrypt-later attacks
- **Zero-Trust Model**: All peers authenticated via libp2p identity keys
- **Replay Protection**: Nonce sliding window prevents replay attacks
- **Secure Key Management**: Keys zeroized on rotation, no persistent storage

### File Transfer Protocol

- **Protocol**: Custom protocol over libp2p request-response
- **Packet Types**: Metadata (type 0), Data Chunk (type 1), End (type 2)
- **Chunk Size**: 64KB default (configurable)
- **Verification**: SHA-256 hash verification on completion
- **Error Handling**: Automatic retry and error reporting

## Verification Status

All verification phases completed successfully:

- ✅ **PHASE 0**: Repo discovery – PASS
- ✅ **PHASE 1**: Rust workspace build & test – PASS
- ✅ **PHASE 2**: Web stack integration – PASS
- ✅ **PHASE 3**: Encrypted tunnel & crypto – PASS
- ✅ **PHASE 4**: File transfer – PASS (fully working end-to-end)
- ✅ **PHASE 5**: VPN mode – PASS
- ✅ **PHASE 6**: Logging & observability – PASS
- ✅ **PHASE 7**: Documentation – PASS

## Build & Test Commands

```bash
# Clean build
cargo clean
cargo build --release -p cryprq

# Run tests
cargo test

# Lint and format
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

## Deployment

### Docker Compose (Recommended)

```bash
# Web stack
docker compose -f docker-compose.web.yml up --build

# VPN stack
docker compose -f docker-compose.vpn.yml up --build
```

### Local Development

```bash
# Build Rust binary
cargo build --release -p cryprq

# Start web server
cd web && npm install && node server/server.mjs
```

## Known Limitations

- **Concurrent Transfers**: Currently supports one file transfer at a time per peer
- **Large Files**: Fixed chunk size (64KB) - optimization for very large files planned
- **TLS Wrapping**: Additional transport security (TLS) not yet implemented
- **DNS-over-TLS**: DNS resolution over encrypted tunnel not yet available
- **KAT Vectors**: Known Answer Test vectors for cryptographic verification planned

## Future Enhancements

- TLS wrapping for additional transport security
- DNS-over-TLS integration
- Known Answer Test (KAT) vectors
- Large file transfer optimization
- Concurrent file transfer support
- Web UI enhancements for file transfer management

## Documentation

- **README.md**: Updated with file transfer commands and examples
- **docs/VERIFICATION_CHECKLIST.md**: Complete verification checklist with file transfer section
- **docs/OPERATOR_LOGS.md**: File transfer log interpretation guide
- **CHANGELOG.md**: Detailed changelog with all changes

## Support

- **Issues**: https://github.com/codethor0/cryprq/issues
- **Email**: codethor@gmail.com
- **Security**: Responsible disclosure to codethor@gmail.com

## Acknowledgments

This release represents significant progress in post-quantum VPN technology. Special thanks to the Rust and libp2p communities for excellent tooling and documentation.

---

**CrypRQ v1.0.0-web-only** - Post-quantum, zero-trust VPN with encrypted file transfer.

