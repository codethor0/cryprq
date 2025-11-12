# Post-Quantum Encryption User Guide

## What is Post-Quantum Encryption?

Post-quantum encryption protects your data against future quantum computer attacks. Traditional encryption methods (like RSA, ECC) can be broken by quantum computers using Shor's algorithm. Post-quantum cryptography uses algorithms that remain secure even against quantum computers.

## CrypRQ's Approach

CrypRQ uses a **hybrid** approach combining:

- **ML-KEM (Kyber768)**: Post-quantum key exchange algorithm (NIST standard)
- **X25519**: Classical elliptic curve cryptography

This provides **defense-in-depth** - even if one algorithm is compromised, the other provides protection.

## Enabling/Disabling Post-Quantum Encryption

### Desktop GUI

1. Open CrypRQ
2. Navigate to **Settings** → **Security**
3. Toggle **"Post-Quantum Encryption"**
   -  **Enabled** (recommended): ML-KEM + X25519 hybrid
   -  **Disabled**: X25519-only (not recommended)

### Mobile App

1. Open CrypRQ Mobile
2. Navigate to **Settings** → **Security**
3. Toggle **"Post-Quantum Encryption"**

### CLI

```bash
## Post-quantum enabled (default)
./cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1

## Disable post-quantum (not recommended)
./cryprq --no-post-quantum --listen /ip4/0.0.0.0/udp/9999/quic-v1
```

## Why Keep It Enabled?

 **Future-proof**: Protects against store-now-decrypt-later attacks  
 **Defense-in-depth**: Multiple layers of security  
 **Minimal overhead**: Optimized implementation  
 **Industry standard**: ML-KEM is a NIST-approved algorithm  

## When Might You Disable It?

 **Not recommended**, but you might disable if:
- Testing compatibility with peers that don't support ML-KEM
- Debugging connection issues
- Performance profiling (though overhead is minimal)

**Important**: Disabling post-quantum encryption reduces your security posture. Only disable if absolutely necessary.

## Key Rotation

CrypRQ automatically rotates encryption keys every **5 minutes** (configurable). This ensures:
- Forward secrecy
- Protection against key compromise
- Automatic cleanup of expired keys

## Technical Details

- **ML-KEM**: Kyber768 variant (1184-byte public keys, 2400-byte secret keys)
- **X25519**: Elliptic curve Diffie-Hellman over Curve25519
- **KDF**: BLAKE3 for key derivation
- **Encryption**: ChaCha20-Poly1305 AEAD

## Learn More

- [Security Model](../security.md)
- [Open Quantum Safe Project](https://openquantumsafe.org/)
- [NIST Post-Quantum Cryptography](https://csrc.nist.gov/projects/post-quantum-cryptography)

