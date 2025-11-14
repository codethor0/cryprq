# CrypRQ v1.0.1-web-preview — Announcement Snippets

Ready-to-use copy for public announcements.

---

## Short "Research-y" Tweet/X

```
Shipped: CrypRQ v1.0.1 – web-only, test-mode preview

A post-quantum-aware encrypted record & file-transfer layer in Rust.

✅ Record-centric design
✅ AEAD + HKDF key schedule
✅ ML-KEM-768 KATs & property tests
✅ Web UI for file transfer over encrypted records

Whitepaper + code: https://github.com/codethor0/cryprq
```

---

## LinkedIn-style Blurb

```
I just published the CrypRQ v1.0.1 web-only preview — a post-quantum-aware encrypted record and file-transfer layer written in Rust.

This preview focuses on:

• A clean, AEAD-based record layer with HKDF-derived keys
• Tested nonce construction, replay windows, and file-transfer protocol
• A web UI that drives real encrypted file transfers end-to-end

It's explicitly test-mode only (static keys, no live handshake yet), but it documents the full cryptographic design and prepares the ground for a hybrid post-quantum handshake.

Whitepaper & code are here: https://github.com/codethor0/cryprq

Feedback from cryptographers, protocol designers, and systems engineers is very welcome.
```

---

## Technical Blog Post Opening

```
# Introducing CrypRQ v1.0.1: A Post-Quantum-Aware Encrypted Record Layer

Today we're releasing CrypRQ v1.0.1-web-preview, a web-only, test-mode preview of a post-quantum-aware encrypted record and file-transfer layer built in Rust.

## Why CrypRQ?

Modern encrypted transport protocols like TLS 1.3 and QUIC provide strong security, but they carry considerable complexity and tight coupling to specific protocol stacks. CrypRQ explores a different axis: a record-layer-centric, post-quantum-aware encrypted transport, optimized for file transfer and tunnel-style networking.

## What's in v1.0.1?

This preview release focuses on the core cryptographic and record-layer design:

- **Record-layer abstraction**: 20-byte header format with epoch, stream ID, sequence number, and message type
- **AEAD encryption**: ChaCha20-Poly1305 with TLS 1.3-style nonce construction
- **HKDF key schedule**: Labeled derivations for epoch keys, traffic keys, and directional IVs
- **Post-quantum awareness**: ML-KEM-768-class KEM integration with hybrid-handshake KATs and property tests
- **File transfer protocol**: Chunked transfer over encrypted records with replay protection
- **Web stack**: Local UI for end-to-end encrypted file transfer

## Important: Test Mode Only

This release is explicitly **not production-ready**. It uses static symmetric keys, has no live handshake, and includes "both sides initiator" simplifications for testing. It's intended for localhost/lab environments and design review.

## What's Next?

The next milestone will include:
- Full hybrid handshake (ECDH + ML-KEM) on the wire
- Peer identity binding
- Production-oriented security profile

Read the full whitepaper: https://github.com/codethor0/cryprq/blob/main/docs/WHITEPAPER.md
```

---

## Academic/Research Context

```
CrypRQ v1.0.1-web-preview: A Post-Quantum-Aware Encrypted Record and File Transfer Layer

We present CrypRQ, a record-layer-centric encrypted transport protocol designed with post-quantum security in mind. The v1.0.1-web-preview demonstrates:

• A clean separation between cryptographic record/traffic layer, node/tunnel logic, and web UI workflows
• AEAD-based record encryption (ChaCha20-Poly1305) with epoch-based key rotation
• HKDF key derivation with domain separation for handshake, traffic, and directional keys
• ML-KEM-768-class post-quantum KEM integration (prepared for hybrid handshake)
• File transfer protocol with replay protection and nonce overflow handling

The current release operates in test mode (static keys, no live handshake) but provides a complete reference implementation and whitepaper documenting the cryptographic design.

Repository: https://github.com/codethor0/cryprq
Whitepaper: https://github.com/codethor0/cryprq/blob/main/docs/WHITEPAPER.md
Citation: See CITATION.cff in repository root
```

---

## Reddit / Hacker News Style

```
**CrypRQ v1.0.1-web-preview: Post-Quantum-Aware Encrypted Record Layer**

I've released a web-only preview of CrypRQ, a post-quantum-aware encrypted record and file-transfer layer written in Rust.

**What it does:**
- Record-layer abstraction with AEAD encryption (ChaCha20-Poly1305)
- HKDF key schedule with epoch-based rotation
- ML-KEM-768 integration (prepared for hybrid handshake)
- File transfer protocol over encrypted records
- Web UI for end-to-end encrypted file transfer

**Current status:**
Test-mode only (static keys, no live handshake). Suitable for localhost/lab testing and design review. Full whitepaper documents the cryptographic design.

**Why:**
Explores a record-layer-centric approach separate from TLS/QUIC complexity, with post-quantum security built in from the start.

GitHub: https://github.com/codethor0/cryprq
Whitepaper: https://github.com/codethor0/cryprq/blob/main/docs/WHITEPAPER.md

Feedback welcome from cryptographers, protocol designers, and systems engineers.
```

