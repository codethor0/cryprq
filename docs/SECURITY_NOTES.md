# Security Notes for CrypRQ v1.0.1 Testing

## IMPORTANT: Current Implementation Status

### Testing vs Production

The current CrypRQ v1.0.1 implementation is **suitable for functional and protocol-level testing** but **NOT for production use** until the following security requirements are met.

---

## MUST FIX Before Production

### 1. Test-Mode Key Direction Hack

**Location:** `node/src/lib.rs` (`Tunnel::recv_record`)

**Current Code:**
```rust
// NOTE: In test mode, both sides assume initiator role, so sender encrypts with keys_outbound (ir)
// and receiver must decrypt with keys_outbound (ir), not keys_inbound (ri)
let keys = self.keys_outbound.read()... // Test-mode workaround
```

**Required Fix:**
- Implement proper handshake with role negotiation (initiator vs responder)
- Initiator encrypts with `key_ir`, decrypts with `key_ri`
- Responder encrypts with `key_ri`, decrypts with `key_ir`
- Remove test-mode symmetric key + initiator-only mapping

**Impact:** Current test-mode hack allows testing but breaks spec compliance. Production must use proper role-based key directions.

### 2. Hardcoded Test Keys

**Location:** `cli/src/main.rs` (handle_send_file, handle_receive_file)

**Current Code:**
```rust
let local_sk = [0x01; 32];
let peer_pk = [0x02; 32];
let peer_identity_key = [0x03; 32];
let peer_signature = [0x04; 64];
```

**Required Fix:**
- Replace with real ML-KEM + X25519 hybrid key exchange
- Generate ephemeral keys per connection
- Use proper key derivation from handshake

**Impact:** Without this fix, all connections use predictable keys, making encryption ineffective.

---

### 2. Missing Real Handshake

**Location:** `node/src/lib.rs` (create_tunnel_with_output_dir)

**Current Code:**
```rust
// TODO: Replace with CrypRQ handshake (CRYPRQ_CLIENT_HELLO/SERVER_HELLO/CLIENT_FINISH)
let peer_public = PublicKey::from(*peer_pk);
let shared_secret = EphemeralSecret::random_from_rng(OsRng).diffie_hellman(&peer_public);
```

**Required Fix:**
- Implement CRYPRQ_CLIENT_HELLO message
- Implement CRYPRQ_SERVER_HELLO message
- Implement CRYPRQ_CLIENT_FINISH message
- Wire ML-KEM (Kyber768) + X25519 hybrid key exchange
- Derive keys from handshake secrets

**Impact:** Without this fix, there's no authenticated key exchange, making the protocol vulnerable to MitM attacks.

---

### 3. No Peer Identity Verification

**Location:** `node/src/lib.rs` (verify_peer_identity)

**Current Status:** Function exists but uses placeholder verification.

**Required Fix:**
- Implement Ed25519 signature verification
- Verify peer identity matches expected peer ID
- Reject connections from unknown peers (if using allowlist)

**Impact:** Without this fix, any peer can connect, making the system vulnerable to unauthorized access.

---

## Safe for Testing

The following components are **safe to test** and **correctly implemented**:

### Record Layer Structure
- 20-byte header with correct field sizes
- Proper encoding/decoding
- Message type routing

### Cryptographic Primitives
- HKDF key derivation (correct salt and labels)
- ChaCha20-Poly1305 AEAD encryption
- TLS 1.3-style nonce construction

### Epoch Management
- u8 epoch with modulo 256 wrapping
- Epoch-scoped key derivation
- Sequence counter reset on epoch change

### File Transfer Protocol
- Stream ID allocation
- File metadata and chunk handling
- SHA-256 hash verification

---

## Testing Recommendations

### Functional Testing
 **Safe to test:**
- File transfer correctness
- Record layer behavior
- Epoch rotation
- Sequence number management
- Concurrent transfers
- Large file handling

### Security Testing
 **Limited value until fixes:**
- Key exchange (uses placeholder)
- Peer authentication (uses placeholder)
- MitM resistance (not implemented)

### Performance Testing
 **Safe to test:**
- Throughput
- Latency
- Resource usage
- Scalability

---

## Migration Path to Production

### Phase 1: Complete Handshake Implementation
1. Implement CRYPRQ_CLIENT_HELLO / SERVER_HELLO / CLIENT_FINISH
2. Wire ML-KEM + X25519 hybrid key exchange
3. Derive keys from handshake secrets

### Phase 2: Identity & Authentication
1. Implement Ed25519 peer identity scheme
2. Verify peer signatures
3. Add peer allowlist/denylist (optional)

### Phase 3: Remove Test Code
1. Remove hardcoded keys from CLI
2. Remove placeholder handshake code
3. Add proper error handling for authentication failures

### Phase 4: Security Audit
1. Review key management
2. Review nonce handling
3. Review replay protection
4. Review rate limiting

---

## Current Security Posture

| Component | Status | Risk Level |
|-----------|--------|------------|
| Record Layer | Correct | Low |
| Encryption (AEAD) | Correct | Low |
| Key Derivation (HKDF) | Correct | Low |
| Handshake | Placeholder | **HIGH** |
| Peer Auth | Placeholder | **HIGH** |
| Key Management | Hardcoded | **HIGH** |

---

## Recommendations

1. **For Development/Testing:** Current implementation is fine for functional validation.

2. **For Production:** Do NOT deploy until all "MUST FIX" items are addressed.

3. **For Security Review:** Focus on handshake and authentication implementation once complete.

4. **For Compliance:** Document that current version is "pre-production" and requires security fixes.

---

## References

- Protocol Specification: `cryp-rq-protocol-v1.md`
- Test Matrix: `docs/TEST_MATRIX.md`
- Master QA Prompt: `docs/MASTER_QA_PROMPT.md`

