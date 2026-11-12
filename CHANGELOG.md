# Changelog

All notable changes to CrypRQ will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0-web-only] - 2025-11-14

### Added
- **File Transfer Over Encrypted Tunnel**: Secure file transfer with SHA-256 integrity verification
  - CLI commands: `send-file` and `receive-file`
  - Chunked transfer protocol with automatic reassembly
  - End-to-end encryption using ML-KEM + X25519 hybrid handshake
  - Hash verification ensures data integrity
- **Web UI File Transfer**: File transfer integration in web interface
  - Upload files via web UI
  - Real-time transfer progress and status
  - Backend integration with core file transfer pipeline

### Fixed
- **libp2p Request-Response Protocol**: Fixed codec implementation to properly handle protocol negotiation
  - `PacketCodec` now always length-prefixes requests for libp2p's request-response protocol
  - Fixed swarm event loop to properly process protocol negotiation events
  - Ensured swarm lock is released immediately after `send_request()` calls
  - Added `tokio::task::yield_now()` to allow event loop to process protocol negotiation
  - Added appropriate delays to allow protocol negotiation to complete
- **Event Loop Blocking**: Fixed premature event loop exit that prevented file transfer completion
  - Event loop now waits for all expected responses before exiting
  - Proper timeout handling for file transfer operations

### Changed
- Improved logging for file transfer operations
  - Reduced verbosity of chunk-level logs (now debug level)
  - Clearer success/failure messages for file transfer
- Enhanced error handling in file transfer callback

### Known Limitations / Future Enhancements
- TLS wrapping for additional transport security
- DNS-over-TLS integration
- Known Answer Test (KAT) vectors for cryptographic verification
- Large file transfer optimization (currently uses fixed chunk size)
- Concurrent file transfer support (currently one transfer at a time per peer)

## [0.9.0] - 2025-11-01

### Added
- Initial web-only CrypRQ release
- Hybrid ML-KEM (Kyber768-compatible) + X25519 handshake
- Five-minute automatic key rotation
- Web UI with React + TypeScript frontend
- Docker Compose deployment configurations
- Real-time log streaming in web UI
- VPN mode with TUN interface support

### Security
- Post-quantum cryptography using ML-KEM
- Zero-trust peer authentication via libp2p identity keys
- Secure key zeroization on rotation
- Replay attack protection with nonce sliding window

