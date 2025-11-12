# Post-Quantum Cryptography Algorithms

## Overview

CrypRQ supports multiple post-quantum cryptography algorithms, with a focus on NIST-standardized algorithms. The current implementation uses ML-KEM (Kyber768) for key exchange, with plans for additional algorithms.

## Supported Algorithms

### Key Exchange

#### ML-KEM 768 (Kyber768-compatible) ✅ **Implemented**
- **Status**: Production-ready
- **Security Level**: Level 3 (equivalent to AES-192)
- **Public Key Size**: 1184 bytes
- **Secret Key Size**: 2400 bytes
- **Ciphertext Size**: 1088 bytes
- **Library**: `pqcrypto-mlkem`
- **NIST Status**: Standardized (FIPS 203)

**Use Case**: Default key exchange algorithm. Provides strong post-quantum security with reasonable key sizes.

#### ML-KEM 1024 (Planned)
- **Status**: Planned
- **Security Level**: Level 5 (equivalent to AES-256)
- **Public Key Size**: 1568 bytes
- **Secret Key Size**: 3168 bytes
- **Use Case**: Higher security requirements

#### X25519 (Classical) ⚠️ **Fallback Only**
- **Status**: Supported as fallback
- **Security**: Classical (not post-quantum)
- **Use Case**: Compatibility fallback (not recommended)

### Signatures

#### Ed25519 (Classical) ✅ **Implemented**
- **Status**: Production-ready
- **Security**: Classical (not post-quantum)
- **Use Case**: Current signature algorithm

#### Dilithium3 (Planned)
- **Status**: Planned
- **Security Level**: Level 3
- **Public Key Size**: ~1952 bytes
- **Signature Size**: ~3309 bytes
- **NIST Status**: Standardized (FIPS 204)
- **Use Case**: Post-quantum signatures

#### SPHINCS+ (Planned)
- **Status**: Planned
- **Security Level**: Level 3
- **Public Key Size**: 64 bytes
- **Signature Size**: ~7856 bytes
- **NIST Status**: Standardized (FIPS 205)
- **Use Case**: Hash-based backup signatures

## Algorithm Selection

### Default Suite
- **Key Exchange**: ML-KEM 768 + X25519 (hybrid)
- **Signatures**: Ed25519
- **Rationale**: Balanced security and performance

### High Security Suite (Planned)
- **Key Exchange**: ML-KEM 1024 + X25519 (hybrid)
- **Signatures**: Dilithium3
- **Rationale**: Maximum security, larger keys

### Legacy Suite (Not Recommended)
- **Key Exchange**: X25519-only
- **Signatures**: Ed25519
- **Rationale**: Compatibility only, not post-quantum

## Implementation Status

| Algorithm | Status | Notes |
|-----------|--------|-------|
| ML-KEM 768 | ✅ Implemented | Default key exchange |
| ML-KEM 1024 | 📋 Planned | Higher security option |
| Dilithium3 | 📋 Planned | Post-quantum signatures |
| SPHINCS+ | 📋 Planned | Hash-based signatures |
| Ed25519 | ✅ Implemented | Current signatures |
| X25519 | ✅ Implemented | Hybrid handshake |

## Performance Considerations

### ML-KEM 768
- **Key Generation**: ~1-2ms
- **Encapsulation**: ~1-2ms
- **Decapsulation**: ~1-2ms
- **Memory**: ~5KB per keypair

### ML-KEM 1024 (Estimated)
- **Key Generation**: ~2-3ms
- **Encapsulation**: ~2-3ms
- **Decapsulation**: ~2-3ms
- **Memory**: ~7KB per keypair

### Dilithium3 (Estimated)
- **Key Generation**: ~2-3ms
- **Signing**: ~1-2ms
- **Verification**: ~1-2ms
- **Memory**: ~6KB per keypair

## Migration Path

### Current (v1.0.1)
- ML-KEM 768 + X25519 hybrid
- Ed25519 signatures

### Future (v1.1.0+)
- Add ML-KEM 1024 option
- Add Dilithium3 signatures
- Add SPHINCS+ signatures
- Algorithm selection UI

## References

- [NIST Post-Quantum Cryptography](https://csrc.nist.gov/projects/post-quantum-cryptography)
- [FIPS 203: ML-KEM](https://csrc.nist.gov/publications/detail/fips/203/final)
- [FIPS 204: Dilithium](https://csrc.nist.gov/publications/detail/fips/204/final)
- [FIPS 205: SPHINCS+](https://csrc.nist.gov/publications/detail/fips/205/final)
- [Open Quantum Safe](https://openquantumsafe.org/)

