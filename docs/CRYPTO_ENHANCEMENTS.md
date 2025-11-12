# Cryptographic Enhancements Guide

## Overview

CrypRQ implements advanced cryptographic features beyond basic encryption to provide comprehensive security and privacy protection.

## Implemented Features

###  Post-Quantum Cryptography

#### Key Exchange
- **ML-KEM 768 + X25519 Hybrid**: Default key exchange (implemented)
- **Post-Quantum Pre-Shared Keys (PPKs)**: Enhanced peer authentication (implemented)
- **Five-minute Key Rotation**: Automatic with secure zeroization (implemented)

#### Data Encryption
- **ChaCha20-Poly1305**: Current data-plane cipher (implemented)
- **Post-Quantum Data Encryption**: Framework ready for PQ cipher integration

###  Traffic Analysis Resistance

#### Packet Padding
- **Configurable Padding**: Pad packets to target size (implemented)
- **Random Padding**: Prevents packet size fingerprinting (implemented)
- **Configurable**: `PaddingConfig` allows customization

#### Constant-Rate Traffic
- **TrafficShaper**: Constant-rate traffic generation (implemented)
- **Jitter Injection**: Adds randomness to avoid perfect constant-rate detection (implemented)
- **Configurable Rate**: Adjustable packets per second

###  DNS Protection

#### DNS-over-HTTPS (DoH)
- **Encrypted DNS Queries**: DoH support (implemented)
- **Default Provider**: Cloudflare DNS (configurable)
- **Timeout Protection**: Configurable query timeouts

#### DNS-over-TLS (DoT)
- **TLS-Encrypted DNS**: DoT support (framework ready)
- **Server Selection**: Configurable DoT servers

###  Transport Security

#### QUIC Protocol
- **libp2p QUIC**: Low-latency, secure transport (implemented)
- **Built-in Encryption**: QUIC provides encryption by default

#### TLS 1.3
- **Control Plane**: TLS 1.3 framework for control communications (implemented)
- **Server/Client Support**: Both server and client modes available
- **Certificate Management**: Configurable certificate paths

###  Advanced Authentication

#### Zero-Knowledge Proofs
- **ZK Proof Framework**: ZKP support for peer authentication (framework ready)
- **Privacy-Preserving**: Authenticate without revealing secrets
- **Hash-Based**: Simplified implementation (can be extended with SNARKs)

###  Forward Secrecy

#### Perfect Forward Secrecy (PFS)
- **Ephemeral Keys**: Each session uses unique keys (implemented)
- **Key Rotation**: Keys rotate every 5 minutes (implemented)
- **Key Zeroization**: Old keys are securely erased (implemented)
- **No Key Derivation**: Keys are not derived from previous keys (ensures PFS)

## Configuration

### Padding Configuration

```rust
use node::PaddingConfig;

let config = PaddingConfig {
    min_packet_size: 64,
    max_packet_size: 1500,
    target_packet_size: 512,
    constant_rate: false,
    constant_rate_interval_ms: 100,
};
```

### DNS Configuration

```rust
use node::DnsConfig;

let config = DnsConfig {
    use_doh: true,
    use_dot: false,
    doh_endpoint: Some("https://cloudflare-dns.com/dns-query".to_string()),
    dot_server: Some("1.1.1.1:853".to_string()),
    timeout: Duration::from_secs(5),
};
```

### TLS Configuration

```rust
use node::TlsConfig;

let config = TlsConfig {
    enabled: true,
    cert_path: Some("/path/to/cert.pem".to_string()),
    key_path: Some("/path/to/key.pem".to_string()),
    ca_cert_path: Some("/path/to/ca.pem".to_string()),
    require_client_auth: false,
};
```

## Usage Examples

### Enable Packet Padding

```rust
use node::{Tunnel, PaddingConfig};

// Configure padding
let padding_config = PaddingConfig {
    target_packet_size: 512,
    ..Default::default()
};

// Padding is automatically applied in Tunnel::send_packet()
```

### Use DNS-over-HTTPS

```rust
use node::{resolve_hostname, DnsConfig};

let config = DnsConfig::default();
let ip = resolve_hostname("example.com", &config).await?;
```

### Use TLS 1.3 for Control Plane

```rust
use node::{TlsServer, TlsConfig};

let mut server = TlsServer::new(TlsConfig::default());
server.listen("127.0.0.1:8443").await?;
let stream = server.accept().await?;
```

### Generate Zero-Knowledge Proof

```rust
use cryprq_crypto::{generate_proof, verify_proof};

let secret = [1u8; 32];
let challenge = [2u8; 32];
let public_info = [3u8; 32];

let proof = generate_proof(&secret, &challenge);
let valid = verify_proof(&proof, &challenge, &public_info);
```

## Security Considerations

### Traffic Analysis Resistance

- **Padding**: Helps prevent packet size analysis
- **Constant Rate**: Makes traffic patterns harder to analyze
- **Jitter**: Prevents perfect constant-rate detection

### DNS Protection

- **DoH/DoT**: Prevents DNS surveillance and manipulation
- **Timeout**: Prevents DNS-based DoS attacks
- **Fallback**: System DNS as last resort (not recommended)

### Forward Secrecy

- **Ephemeral Keys**: Each session uses unique keys
- **No Key Derivation**: Keys are not derived from previous keys
- **Secure Erasure**: Old keys are zeroized

## Future Enhancements

### Planned Features

1. **Post-Quantum Data Ciphers**
   - AES-256-GCM-SIV (PQ-resistant)
   - XChaCha20-Poly1305 (larger nonce space)

2. **Enhanced ZK Proofs**
   - SNARK-based proofs (using arkworks)
   - Bulletproofs for range proofs

3. **Secure Multi-Party Computation (SMPC)**
   - Multi-party key generation
   - Distributed trust

4. **Metadata Minimization**
   - Header compression
   - Traffic pattern obfuscation

## References

- [NIST Post-Quantum Cryptography](https://csrc.nist.gov/projects/post-quantum-cryptography)
- [DNS-over-HTTPS RFC 8484](https://tools.ietf.org/html/rfc8484)
- [DNS-over-TLS RFC 7858](https://tools.ietf.org/html/rfc7858)
- [TLS 1.3 RFC 8446](https://tools.ietf.org/html/rfc8446)
- [Zero-Knowledge Proofs](https://en.wikipedia.org/wiki/Zero-knowledge_proof)

