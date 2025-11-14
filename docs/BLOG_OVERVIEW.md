# CrypRQ: Post-Quantum Secure Communication for the Modern Web

**A technical overview of CrypRQ v1.0.1-web-preview**

---

## What is CrypRQ?

CrypRQ is a post-quantum-ready encrypted communication protocol that provides secure tunnels and file transfer capabilities over peer-to-peer connections. Built with modern cryptographic primitives and designed for the web, CrypRQ combines classical and post-quantum algorithms to ensure security against both current and future threats.

**Key Features:**
- 🔐 **Hybrid Post-Quantum Security**: ML-KEM (Kyber768) + X25519 key exchange
- 🔄 **Automatic Key Rotation**: Ephemeral keys refresh every 5 minutes
- 📦 **Stream Multiplexing**: Multiple concurrent data streams over a single session
- 🌐 **Web-First Architecture**: React + TypeScript frontend, Docker deployment
- 📁 **Secure File Transfer**: End-to-end encrypted file transfer with integrity verification

---

## Why Post-Quantum?

The cryptographic algorithms we rely on today—like RSA and elliptic curve cryptography—are vulnerable to attacks from quantum computers. While practical quantum computers don't exist yet, the threat is real enough that organizations like NIST are already standardizing post-quantum algorithms.

CrypRQ takes a **hybrid approach**: it combines classical cryptography (X25519) with post-quantum cryptography (ML-KEM/Kyber768). This means:
- ✅ Security against classical attacks (via X25519)
- ✅ Security against quantum attacks (via ML-KEM)
- ✅ Defense-in-depth: both algorithms must be broken for the session to be compromised

---

## How It Works

### 1. Record Layer

All data in CrypRQ is encapsulated in **records** with a 20-byte header:

```
┌─────────────────────────────────────────┐
│ Version │ Epoch │ Stream ID │ Sequence  │
│ Msg Type│ Flags │ Encrypted Payload      │
└─────────────────────────────────────────┘
```

Each record includes:
- **Epoch**: Identifies which generation of keys to use (rotates every 5 minutes)
- **Stream ID**: Allows multiple concurrent streams (file transfer, VPN, control)
- **Sequence Number**: Prevents replay attacks and ensures ordering
- **Message Type**: Routes to the correct handler (FILE_META, FILE_CHUNK, VPN_PACKET, etc.)

### 2. Encryption

CrypRQ uses **ChaCha20-Poly1305** for authenticated encryption:
- **ChaCha20**: Fast, secure stream cipher
- **Poly1305**: Provides authentication (detects tampering)

Nonces are constructed using TLS 1.3-style XOR to ensure uniqueness:
```
nonce = static_iv XOR sequence_number
```

### 3. Key Rotation

Every 5 minutes, CrypRQ automatically rotates keys:
1. Epoch counter increments
2. New keys derived from master secret using HKDF
3. Old keys securely zeroized
4. No handshake required—rotation is seamless

This provides **forward secrecy**: even if long-term keys are compromised, past communications remain secure.

### 4. File Transfer

CrypRQ implements a secure file transfer protocol:
1. **FILE_META**: Sender sends file metadata (filename, size, SHA-256 hash)
2. **FILE_CHUNK**: File data sent in 64KB chunks
3. **Verification**: Receiver verifies SHA-256 hash matches

All transfer happens over encrypted records, ensuring confidentiality and integrity.

---

## Architecture

### Backend (Rust)

- **Record Layer**: 20-byte header format with epoch-based key rotation
- **libp2p QUIC**: Transport layer for peer-to-peer communication
- **File Transfer Manager**: Handles incoming/outgoing transfers
- **Key Rotation**: Automatic 5-minute key refresh

### Frontend (React + TypeScript)

- **Connection Management**: Start listener/dialer modes
- **File Transfer UI**: Web-based interface for sending/receiving files
- **Real-Time Logs**: Structured event streaming (handshake, rotation, transfers)

### Deployment

- **Docker Compose**: Single command deployment
- **Test Mode**: Static keys for testing (not production-ready)
- **Web Accessible**: Runs on localhost:8787

---

## Current Status: v1.0.1-web-preview

**What's Working:**
- ✅ Record layer with 20-byte header format
- ✅ Epoch-based key rotation (8-bit epoch counter)
- ✅ TLS 1.3-style nonce construction
- ✅ HKDF-based key derivation
- ✅ File transfer protocol (FILE_META, FILE_CHUNK, FILE_ACK)
- ✅ Web UI for connection management
- ✅ Docker-based deployment

**What's Coming:**
- 🔄 Hybrid handshake (ML-KEM + X25519)
- 🔄 Peer identity and authentication
- 🔄 Proper role-based key directions
- 🔄 Replay protection window
- 🔄 Production hardening

**Important:** The current preview uses **test mode** (static keys, no handshake, no authentication). It's suitable for testing and protocol exploration, but **not for production use**.

---

## Getting Started

### Quick Start (Docker)

```bash
git clone https://github.com/codethor0/cryprq.git
cd cryprq

# Build and start web stack
docker compose -f docker-compose.web.yml up --build

# Open http://localhost:8787 in your browser
```

### CLI (For Testing)

```bash
# Build
cargo build --release -p cryprq

# Listener (Terminal 1)
./target/release/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1

# Dialer (Terminal 2)
./target/release/cryprq --peer /ip4/127.0.0.1/udp/9999/quic-v1
```

---

## Use Cases

### Secure File Transfer

CrypRQ provides end-to-end encrypted file transfer with:
- Integrity verification (SHA-256)
- Chunked transfer for large files
- Real-time progress monitoring

### Encrypted Tunnels

The record layer can carry arbitrary data streams:
- VPN packets (planned)
- Generic application data
- Control messages

### Protocol Exploration

The v1.0.1-web-preview enables:
- Testing cryptographic design
- Validating record layer format
- Exploring key rotation mechanisms

---

## Security Considerations

### Current Limitations

The preview release operates in **test mode**:
- Static keys (no key exchange)
- No handshake (missing CLIENT_HELLO / SERVER_HELLO / CLIENT_FINISH)
- No authentication (no peer identity verification)
- Test-mode workarounds

**These limitations MUST be addressed before production use.**

### Planned Enhancements

Future releases will include:
- Hybrid handshake (ML-KEM + X25519)
- Peer authentication (Ed25519 signatures)
- Proper key directions (role-based selection)
- Replay protection (sliding window validation)

---

## Learn More

- **Whitepaper**: [`docs/WHITEPAPER.md`](docs/WHITEPAPER.md) — Comprehensive technical overview
- **Protocol Spec**: [`cryp-rq-protocol-v1.md`](cryp-rq-protocol-v1.md) — Complete protocol specification
- **Security Notes**: [`docs/SECURITY_NOTES.md`](docs/SECURITY_NOTES.md) — Security posture and limitations
- **Docker Guide**: [`docs/DOCKER_WEB_GUIDE.md`](docs/DOCKER_WEB_GUIDE.md) — Deployment guide

---

## Contributing

CrypRQ is open source and welcomes contributions! Areas of focus:
- Handshake implementation (ML-KEM + X25519)
- Peer authentication
- Production hardening
- Performance optimization

See the [Development section](README.md#development) in the README for pre-flight checks and CI requirements.

---

## License

CrypRQ is licensed under the [MIT License](LICENSE).

---

**Repository:** https://github.com/codethor0/cryprq  
**Contact:** codethor@gmail.com  
**Version:** v1.0.1-web-preview (Test Mode)

