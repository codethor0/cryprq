# CrypRQ Protocol Specification v1.0.1 - Code Alignment Report

**Date:** November 14, 2025  
**Status:** ⚠️ **ARCHITECTURAL MISMATCH DETECTED**

## Executive Summary

The current CrypRQ implementation uses **libp2p's built-in protocols** (Noise handshake, request-response framing) rather than implementing the **custom CrypRQ record layer** specified in `cryp-rq-protocol-v1.md`. This is an architectural difference that needs to be addressed.

## Critical Discrepancies

### 1. Record Header Structure ❌

**Specification (Section 6.1):**
- 20-byte fixed header with fields:
  - Version (1 byte)
  - Message Type (1 byte)
  - Flags (1 byte)
  - Epoch (1 byte, 8-bit)
  - Stream ID (4 bytes)
  - Sequence Number (8 bytes)
  - Ciphertext Length (4 bytes)

**Implementation:**
- Uses libp2p's `request-response` protocol which handles framing internally
- No custom 20-byte record header structure found
- File transfer packets use a simple 4-byte type prefix (see `p2p/src/file_transfer.rs`)

**Impact:** HIGH - Core protocol structure mismatch

### 2. Epoch Field ❌

**Specification (Section 5.3.1):**
- Epoch is an **8-bit unsigned integer** (modulo 256)
- Included in every record header
- Used for key rotation tracking

**Implementation:**
- Epoch tracked as `u64` counter in metrics (`p2p/src/metrics.rs:164`)
- Not included in record headers
- Used only for logging/metrics, not for key selection

**Impact:** HIGH - Key rotation mechanism doesn't match spec

### 3. Nonce Construction ❌

**Specification (Section 6.2.2):**
- TLS 1.3-style XOR construction:
  ```
  seq_be = 0x00000000 || seq_64_be  // 32 zero bits + 64-bit big-endian seq
  nonce = static_iv XOR seq_be
  ```

**Implementation (`node/src/lib.rs:384-387`):**
```rust
let nonce_bytes = counter.to_le_bytes();
let mut nonce_arr = [0u8; 12];
nonce_arr[..8].copy_from_slice(&nonce_bytes);
```
- Uses little-endian counter directly
- No XOR with static IV
- No 96-bit encoding

**Impact:** HIGH - Nonce construction doesn't match spec

### 4. Key Derivation ❌

**Specification (Section 4.4):**
- Salt: `salt_hs = "cryp-rq v1.0 hs"`
- HKDF labels:
  - `"cryp-rq hs auth"` for handshake auth key
  - `"cryp-rq master secret"` for master secret
  - `"cryp-rq ir key"`, `"cryp-rq ir iv"` for traffic keys
  - `"cryp-rq ri key"`, `"cryp-rq ri iv"` for reverse traffic keys
- Epoch-scoped keys: `"cryp-rq ir key epoch=" || epoch`

**Implementation:**
- Uses BLAKE3 KDF (mentioned in `node/src/lib.rs:12`)
- No explicit HKDF implementation found
- No `salt_hs` or label strings found in codebase
- Key rotation doesn't use epoch-scoped derivation

**Impact:** HIGH - Key derivation doesn't match spec

### 5. Handshake Messages ❌

**Specification (Section 4.2):**
- `CRYPRQ_CLIENT_HELLO` (plaintext)
- `CRYPRQ_SERVER_HELLO` (plaintext)
- `CRYPRQ_CLIENT_FINISH` (plaintext, authenticated)

**Implementation:**
- Uses libp2p's Noise protocol for handshake
- No explicit `CRYPRQ_CLIENT_HELLO`/`CRYPRQ_SERVER_HELLO`/`CRYPRQ_CLIENT_FINISH` messages
- Handshake handled by libp2p's Noise implementation

**Impact:** HIGH - Handshake flow doesn't match spec

### 6. Ciphertext Length Field ❌

**Specification (Section 6.1.1):**
- Ciphertext Length: **4 bytes** (big-endian)

**Implementation:**
- libp2p handles framing internally
- File transfer uses variable-length packets with 4-byte type prefix
- No explicit 4-byte ciphertext length field

**Impact:** MEDIUM - Field size mismatch (if custom record layer implemented)

## Partial Alignments ✓

### 1. Cryptographic Primitives ✓
- ML-KEM (Kyber768): ✅ Used (`crypto/src/hybrid.rs`)
- X25519: ✅ Used (`crypto/src/hybrid.rs`, `node/src/lib.rs`)
- ChaCha20-Poly1305: ✅ Used (`node/src/lib.rs`)
- BLAKE3: ✅ Used (mentioned, but spec requires HKDF)

### 2. Key Rotation Concept ✓
- Periodic key rotation: ✅ Implemented (`p2p/src/lib.rs`)
- 5-minute interval: ✅ Configurable
- Epoch tracking: ✅ Implemented (but as u64, not u8)

### 3. File Transfer ✓
- File metadata, chunks, end packet: ✅ Implemented (`p2p/src/file_transfer.rs`)
- SHA-256 verification: ✅ Implemented

## Recommendations

### Option A: Align Code to Specification (Recommended for Protocol Compliance)

1. **Implement Custom Record Layer**
   - Create `Record` struct with 20-byte header
   - Implement serialization/deserialization
   - Replace libp2p request-response with custom framing

2. **Fix Nonce Construction**
   - Implement TLS 1.3-style XOR: `nonce = static_iv XOR seq_be`
   - Use 96-bit big-endian encoding for sequence number

3. **Implement HKDF Key Derivation**
   - Replace BLAKE3 with HKDF (or use BLAKE3 as HKDF extract/expand)
   - Use exact salt and labels from spec:
     - `salt_hs = "cryp-rq v1.0 hs"`
     - Labels: `"cryp-rq hs auth"`, `"cryp-rq master secret"`, etc.

4. **Implement Handshake Messages**
   - Create `CRYPRQ_CLIENT_HELLO`, `CRYPRQ_SERVER_HELLO`, `CRYPRQ_CLIENT_FINISH` message types
   - Implement handshake flow matching Section 4.2

5. **Fix Epoch**
   - Change epoch from `u64` to `u8`
   - Include epoch in record header
   - Use epoch in key derivation labels

### Option B: Align Specification to Implementation (Faster, but breaks compatibility)

1. **Update Specification**
   - Document libp2p Noise handshake instead of custom messages
   - Document libp2p request-response framing instead of custom records
   - Update nonce construction to match current implementation
   - Update key derivation to use BLAKE3 instead of HKDF
   - Document epoch as u64 counter (not in header)

2. **Risks**
   - Breaks protocol specification independence
   - Makes it harder for other implementations
   - Loses some protocol-level control

## Files Requiring Changes (Option A)

### High Priority
- `p2p/src/lib.rs` - Add custom record layer
- `node/src/lib.rs` - Fix nonce construction
- `crypto/src/lib.rs` - Implement HKDF key derivation
- `p2p/src/packet_forwarder.rs` - Update to use custom records

### Medium Priority
- `p2p/src/file_transfer.rs` - Align with record header structure
- `p2p/src/metrics.rs` - Change epoch to u8
- Create new `p2p/src/handshake.rs` - Implement handshake messages

## Testing Requirements

After alignment, verify:
1. ✅ Record header serialization/deserialization
2. ✅ Nonce construction matches spec
3. ✅ Key derivation produces expected keys
4. ✅ Handshake flow matches Section 4.2
5. ✅ Epoch wraps correctly at 255
6. ✅ Interoperability with spec-compliant implementations

## Conclusion

The current implementation is **functionally working** but uses a **different architecture** than specified in the protocol document. To achieve full protocol compliance, significant refactoring is required. The choice between aligning code to spec vs. aligning spec to code depends on the project's goals:

- **Protocol compliance**: Choose Option A
- **Rapid development**: Choose Option B (but update spec)

**Recommendation:** Given that a protocol specification exists and is intended for interoperability, **Option A (align code to spec)** is recommended for long-term maintainability and compatibility.

