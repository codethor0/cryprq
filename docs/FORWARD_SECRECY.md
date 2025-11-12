# Perfect Forward Secrecy (PFS) Implementation

## Overview

CrypRQ implements Perfect Forward Secrecy (PFS) to ensure that compromise of current session keys does not compromise past session keys.

## Implementation Details

### Key Characteristics

1. **Ephemeral Keys**: Each session uses unique, ephemeral keys
2. **No Key Derivation**: Keys are not derived from previous keys
3. **Secure Zeroization**: Old keys are securely erased from memory
4. **Automatic Rotation**: Keys rotate every 5 minutes (configurable)

### Key Exchange

- **ML-KEM + X25519 Hybrid**: Post-quantum key exchange
- **Ephemeral Secrets**: Each handshake generates new ephemeral secrets
- **No Long-Term Keys**: No long-term keys stored for session encryption

### Key Rotation

```rust
// Keys rotate every 5 minutes (300 seconds)
// Old keys are zeroized immediately after rotation
async fn rotate_once(interval: Duration) {
    let (pk, sk) = kyber_keypair(); // New ephemeral keys
    
    // Replace old keys (old keys are dropped and zeroized)
    guard.replace((pk, sk));
    
    // Cleanup expired PPKs
    ppk_store.cleanup_expired(now);
}
```

### Session Key Derivation

```rust
// Session keys derived from ephemeral DH + peer identity
let shared_secret = EphemeralSecret::random_from_rng(OsRng)
    .diffie_hellman(&peer_public);

// BLAKE3 KDF with peer identity
let mut kdf_input = Vec::with_capacity(32 + 32);
kdf_input.extend_from_slice(shared_secret.as_bytes());
kdf_input.extend_from_slice(peer_identity_key);
let session_key = *blake3::hash(&kdf_input).as_bytes();
```

## Security Properties

### Forward Secrecy ✅

- **Ephemeral Keys**: Each session uses unique keys
- **No Key Storage**: Session keys are not stored long-term
- **Secure Erasure**: Keys are zeroized after use

### Perfect Forward Secrecy ✅

- **No Key Derivation**: Keys are not derived from previous keys
- **Independent Sessions**: Each session is cryptographically independent
- **Compromise Isolation**: Compromise of one session does not affect others

## Verification

### Key Independence

```rust
// Session 1 keys
let session_key_1 = derive_session_key(ephemeral_secret_1, peer_id);

// Session 2 keys (after rotation)
let session_key_2 = derive_session_key(ephemeral_secret_2, peer_id);

// Keys are independent - compromise of session_key_1 does not reveal session_key_2
assert_ne!(session_key_1, session_key_2);
```

### Zeroization

```rust
use zeroize::Zeroize;

// Old keys are automatically zeroized when dropped
impl Drop for SessionKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
```

## Configuration

### Rotation Interval

```bash
## Default: 300 seconds (5 minutes)
CRYPRQ_ROTATE_SECS=300 ./cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1

## Custom: 60 seconds (1 minute)
CRYPRQ_ROTATE_SECS=60 ./cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1
```

### Minimum Rotation Interval

- **Recommended**: 60 seconds minimum
- **Default**: 300 seconds (5 minutes)
- **Maximum**: 3600 seconds (1 hour)

## Threat Model

### Protected Against

✅ **Retroactive Decryption**: Past sessions cannot be decrypted even if current keys are compromised  
✅ **Key Compromise**: Compromise of one session key does not affect other sessions  
✅ **Long-Term Storage Attacks**: No long-term keys stored for session encryption  

### Not Protected Against

⚠️ **Active MitM**: Requires additional authentication (Ed25519 signatures)  
⚠️ **Replay Attacks**: Handled separately with replay window  
⚠️ **Traffic Analysis**: Requires additional padding/shaping (implemented separately)  

## References

- [Forward Secrecy](https://en.wikipedia.org/wiki/Forward_secrecy)
- [Perfect Forward Secrecy](https://en.wikipedia.org/wiki/Forward_secrecy#Perfect_forward_secrecy)
- [NIST Key Management Guidelines](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)

