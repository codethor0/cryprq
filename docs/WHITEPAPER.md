# CrypRQ: A Post-Quantum–Ready Encrypted Record and File Transfer System

**Project:** CrypRQ  
**Version:** v1.0.1-web-preview (Web-only, Test Mode)  
**Reference implementation:** Git tag `v1.0.1-web-preview`  
**Validated demo build:** `main` @ `db0903f`  
**Date:** 2025-11-14

---

## Abstract

CrypRQ is a post-quantum-hybrid secure tunnel and file-transfer protocol designed for peer-to-peer communication. It leverages a hybrid key exchange combining ML-KEM (Kyber768) and X25519 to establish secure, ephemeral session keys, ensuring confidentiality and integrity of data in transit. The protocol operates over QUIC/UDP and provides automatic key rotation every five minutes, forward secrecy, and support for multiple concurrent data streams including file transfer and VPN functionality.

This whitepaper documents the CrypRQ v1.0.1-web-preview architecture, which implements the core record layer and file transfer capabilities in a web-accessible format. The reference implementation demonstrates the protocol's cryptographic design, including the 20-byte record header format, epoch-based key rotation, TLS 1.3-style nonce construction, and HKDF-based key derivation.

> **Security Status (Preview Only)**
> 
> - Static test keys
> - No handshake (no CLIENT_HELLO / SERVER_HELLO / CLIENT_FINISH)
> - No peer identity or authentication
> - Not suitable for production use
> 
> See `docs/SECURITY_NOTES.md` and `docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md` for the authoritative disclaimers.

---

## 1. Introduction

CrypRQ v1.0.1-web-preview is a technical preview of a post-quantum secure communication protocol designed to provide encrypted tunnels and secure file transfer capabilities. The protocol is built on modern cryptographic primitives, combining classical and post-quantum algorithms to ensure security against both current and future threats.

### 1.1 Protocol Goals

The fundamental goal of CrypRQ is to establish a secure, high-performance communication channel between two peers with the following properties:

- **Confidentiality**: All data is encrypted using authenticated encryption with associated data (AEAD)
- **Integrity**: Data tampering is detected through AEAD authentication
- **Post-Quantum Security**: Hybrid key exchange using ML-KEM (Kyber768) + X25519
- **Forward Secrecy**: Ephemeral key rotation every five minutes
- **Stream Multiplexing**: Support for multiple concurrent data streams

### 1.2 Architecture Overview

CrypRQ operates as a layer above transport protocols (primarily QUIC/UDP) to provide a secure channel for application data. The protocol's lifecycle includes:

1. **Cryptographic Handshake**: Hybrid key exchange using ML-KEM + X25519 (planned for future release)
2. **Key Derivation**: HKDF-based derivation of master secret and traffic keys
3. **Record Layer**: 20-byte header format with epoch, stream ID, sequence number, and message type
4. **Key Rotation**: Automatic refresh of traffic keys every five minutes
5. **Data Transfer**: Encrypted records carrying file transfer, VPN packets, or generic data

---

## 2. Cryptographic Design

### 2.1 Record Layer Format

Each CrypRQ record consists of a 20-byte header followed by encrypted payload:

```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Version (u8)  |  Epoch (u8)  |      Stream ID (u32)          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Sequence Number (u64)                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Msg Type (u8) | Flags (u8)   |         Reserved (u16)        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                    Encrypted Payload + Tag                    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

**Header Fields:**
- **Version**: Protocol version (currently 0x01 for v1.0.1)
- **Epoch**: 8-bit epoch counter for key rotation (modulo 256)
- **Stream ID**: 32-bit identifier for multiplexing streams
- **Sequence Number**: 64-bit per-stream sequence counter
- **Message Type**: 8-bit type (VPN_PACKET, FILE_META, FILE_CHUNK, FILE_ACK, DATA, CONTROL)
- **Flags**: 8-bit flags field for future extensions

### 2.2 Encryption and Nonce Construction

CrypRQ uses **ChaCha20-Poly1305** for authenticated encryption. Nonces are constructed using TLS 1.3-style XOR construction:

```
nonce = static_iv XOR (0x00...00 || sequence_number_be)
```

Where:
- `static_iv` is a 12-byte initialization vector derived per direction (ir/ri)
- `sequence_number_be` is the 8-byte big-endian sequence number
- XOR is applied to the last 8 bytes of the IV

This ensures that:
- Same (IV, seq) pair produces the same nonce
- Different sequence numbers produce different nonces
- Replay attacks are detectable via sequence number validation

### 2.3 Key Derivation

Keys are derived using **HKDF** (HMAC-based Key Derivation Function) with SHA-256:

**Traffic Keys (per epoch):**
```
key_ir = HKDF-Expand(master_secret, "cryp-rq epoch N ir key", 32)
iv_ir  = HKDF-Expand(master_secret, "cryp-rq epoch N ir iv", 12)
key_ri = HKDF-Expand(master_secret, "cryp-rq epoch N ri key", 32)
iv_ri  = HKDF-Expand(master_secret, "cryp-rq epoch N ri iv", 12)
```

Where:
- `ir` = initiator-to-responder direction
- `ri` = responder-to-initiator direction
- `N` = epoch number (0-255)

### 2.4 Key Rotation

Keys are rotated every five minutes (configurable via `CRYPRQ_ROTATE_SECS`):

1. Epoch counter increments (modulo 256)
2. New traffic keys derived for the new epoch
3. Old keys are securely zeroized
4. Records include epoch number in header for key selection

---

## 3. File Transfer Protocol

CrypRQ implements a secure file transfer protocol over the encrypted record layer:

### 3.1 Transfer Flow

1. **FILE_META**: Sender sends file metadata (filename, size, SHA-256 hash)
2. **FILE_CHUNK**: Sender sends file data in chunks (default 64KB)
3. **FILE_ACK**: Receiver acknowledges chunks (optional, for flow control)
4. **Verification**: Receiver verifies final SHA-256 hash matches metadata

### 3.2 Message Types

- **FILE_META** (msg_type=3): File metadata packet
- **FILE_CHUNK** (msg_type=4): File data chunk
- **FILE_ACK** (msg_type=5): Chunk acknowledgment
- **CONTROL** (msg_type=6): Control messages (end-of-file, errors)

Each message type is encapsulated in a CrypRQ record with appropriate stream ID and sequence number.

---

## 4. Web Stack Architecture

The v1.0.1-web-preview includes:

### 4.1 Backend

- **Rust binary**: Implements CrypRQ record layer and file transfer
- **libp2p QUIC**: Transport layer for peer-to-peer communication
- **Record layer**: 20-byte header format with epoch-based key rotation
- **File transfer manager**: Handles incoming/outgoing file transfers

### 4.2 Frontend

- **React + TypeScript**: Modern web UI for connection management
- **Real-time logs**: Structured event streaming (handshake, rotation, file transfer)
- **File transfer UI**: Web-based interface for sending/receiving files

### 4.3 Deployment

- **Docker Compose**: Single-command deployment (`docker compose -f docker-compose.web.yml up`)
- **Test mode**: Static keys, no handshake (for testing only)
- **Ports**: Frontend (8787), Backend API (8787), UDP tunnel (9999)

---

## 5. Security Considerations

### 5.1 Current Limitations (v1.0.1-web-preview)

The preview release operates in **test mode** with the following limitations:

- **Static Keys**: Pre-shared keys for testing (no key exchange)
- **No Handshake**: Missing CLIENT_HELLO / SERVER_HELLO / CLIENT_FINISH flow
- **No Authentication**: No peer identity verification
- **Test-Mode Hacks**: Key direction workarounds for testing

**These limitations MUST be addressed before production use.**

### 5.2 Planned Security Enhancements

Future releases will include:

- **Hybrid Handshake**: ML-KEM (Kyber768) + X25519 key exchange
- **Peer Authentication**: Ed25519 signature verification
- **Proper Key Directions**: Role-based key selection (initiator/responder)
- **Replay Protection**: Sliding window nonce validation

### 5.3 Cryptographic Assumptions

CrypRQ assumes:

- **ChaCha20-Poly1305**: Secure AEAD cipher
- **HKDF-SHA256**: Secure key derivation function
- **ML-KEM-768**: Post-quantum key encapsulation (planned)
- **X25519**: Classical elliptic curve Diffie-Hellman (planned)

---

## 6. Implementation Status

### 6.1 Completed (v1.0.1-web-preview)

- ✅ Record layer with 20-byte header format
- ✅ Epoch-based key rotation (8-bit epoch counter)
- ✅ TLS 1.3-style nonce construction
- ✅ HKDF-based key derivation
- ✅ File transfer protocol (FILE_META, FILE_CHUNK, FILE_ACK)
- ✅ Web UI for connection management and file transfer
- ✅ Docker-based deployment
- ✅ Structured logging and observability

### 6.2 In Progress / Planned

- 🔄 Hybrid handshake (ML-KEM + X25519)
- 🔄 Peer identity and authentication
- 🔄 Proper role-based key directions
- 🔄 Replay protection window
- 🔄 VPN mode integration
- 🔄 Production hardening

---

## 7. Performance Characteristics

### 7.1 Record Overhead

- **Header**: 20 bytes per record
- **AEAD Tag**: 16 bytes (Poly1305)
- **Total Overhead**: ~36 bytes per record

### 7.2 Key Rotation Impact

- **Rotation Interval**: 5 minutes (configurable)
- **Rotation Latency**: < 10ms (key derivation + epoch increment)
- **No Handshake Required**: Keys rotate without re-handshake

### 7.3 File Transfer Performance

- **Chunk Size**: 64KB (configurable)
- **Throughput**: Limited by network and encryption overhead
- **Integrity**: SHA-256 verification on completion

---

## 8. Use Cases

### 8.1 Secure File Transfer

CrypRQ provides end-to-end encrypted file transfer with:
- Integrity verification (SHA-256)
- Chunked transfer for large files
- Real-time progress monitoring

### 8.2 Encrypted Tunnels

The record layer can carry arbitrary data streams:
- VPN packets (planned)
- Generic application data
- Control messages

### 8.3 Protocol Exploration

The v1.0.1-web-preview enables:
- Testing cryptographic design
- Validating record layer format
- Exploring key rotation mechanisms

---

## 9. Future Directions

### 9.1 Handshake Implementation

The next major milestone is implementing the full hybrid handshake:
- CLIENT_HELLO with ML-KEM + X25519 public keys
- SERVER_HELLO with responder keys
- CLIENT_FINISH with authentication
- Master secret derivation

### 9.2 Production Hardening

Before production deployment:
- Remove test-mode static keys
- Implement proper peer authentication
- Add replay protection
- Security audit and penetration testing

### 9.3 Protocol Extensions

Future enhancements may include:
- TLS wrapping for compatibility
- DNS-over-TLS integration
- Known Answer Test (KAT) vectors
- Multi-peer support

---

## 10. Conclusion

CrypRQ v1.0.1-web-preview demonstrates a post-quantum-ready encrypted record and file transfer system with automatic key rotation and stream multiplexing. The protocol's cryptographic design combines classical and post-quantum algorithms to ensure security against both current and future threats.

The reference implementation provides a working demonstration of the record layer, key rotation, and file transfer capabilities, while the web stack makes the protocol accessible for testing and exploration. Future releases will complete the handshake implementation and add production-ready security features.

---

## Implementation References

- [`docs/VALIDATION_RUN.md`](VALIDATION_RUN.md) – CLI validation run and reference hash
- [`docs/WEB_VALIDATION_RUN.md`](WEB_VALIDATION_RUN.md) – WEB-1 validation (hash-verified file transfer)
- [`docs/DOCKER_WEB_GUIDE.md`](DOCKER_WEB_GUIDE.md) – Web deployment and image/tag strategy
- [`docs/SECURITY_NOTES.md`](SECURITY_NOTES.md) – Security posture and limitations
- [`docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`](WEB_ONLY_RELEASE_NOTES_v1.0.1.md) – Web-only release summary
- [`cryp-rq-protocol-v1.md`](../cryp-rq-protocol-v1.md) – Complete protocol specification (v1.0.1)
- [`docs/MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`](MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md) – Roadmap for handshake and identity

---

**License:** MIT  
**Repository:** https://github.com/codethor0/cryprq  
**Contact:** codethor@gmail.com

