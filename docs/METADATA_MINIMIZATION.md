# Metadata Minimization

## Overview

CrypRQ implements metadata minimization techniques to reduce the amount of information leaked through network traffic patterns and packet headers.

## Implemented Techniques

###  Packet Padding

- **Random Padding**: Packets are padded with random bytes to prevent size fingerprinting
- **Target Size**: Packets padded to target size (default: 512 bytes)
- **Configurable**: Padding can be customized or disabled

###  Traffic Shaping

- **Constant-Rate Traffic**: Optional constant-rate traffic generation
- **Jitter Injection**: Random delays to avoid perfect constant-rate detection
- **Configurable Rate**: Adjustable packets per second

###  DNS Protection

- **DNS-over-HTTPS**: Encrypted DNS queries prevent DNS metadata leakage
- **DNS-over-TLS**: Alternative encrypted DNS method
- **No DNS Leakage**: DNS queries do not reveal destination hostnames

###  Minimal Headers

- **Compact Protocol**: Minimal protocol overhead
- **No Unnecessary Metadata**: Only essential information in headers
- **Encrypted Headers**: Headers are encrypted along with payload

## Configuration

### Enable Padding

```rust
use node::PaddingConfig;

let config = PaddingConfig {
    target_packet_size: 512,  // Pad to 512 bytes
    min_packet_size: 64,
    max_packet_size: 1500,
    constant_rate: false,
    constant_rate_interval_ms: 100,
};
```

### Enable Constant-Rate Traffic

```rust
use node::TrafficShaper;

let mut shaper = TrafficShaper::new(10.0); // 10 packets/second
shaper.wait_for_slot().await;
```

### Use Encrypted DNS

```rust
use node::{DnsConfig, resolve_hostname};

let config = DnsConfig {
    use_doh: true,
    doh_endpoint: Some("https://cloudflare-dns.com/dns-query".to_string()),
    ..Default::default()
};

let ip = resolve_hostname("example.com", &config).await?;
```

## Metadata Minimized

###  Not Leaked

- **Packet Sizes**: Padding masks actual packet sizes
- **Traffic Patterns**: Constant-rate traffic hides patterns
- **DNS Queries**: DoH/DoT encrypts DNS metadata
- **Timing Information**: Jitter obscures timing patterns

###  Still Visible

- **Connection Existence**: Connection to CrypRQ peer is visible
- **Traffic Volume**: Total traffic volume is visible
- **IP Addresses**: Peer IP addresses are visible (by design for connectivity)

## Future Enhancements

### Planned Features

1. **Header Compression**: Compress protocol headers
2. **Traffic Obfuscation**: Additional traffic pattern obfuscation
3. **Metadata Encryption**: Encrypt all metadata fields
4. **Cover Traffic**: Generate cover traffic to mask real traffic

## Best Practices

1. **Enable Padding**: Always enable packet padding for maximum privacy
2. **Use DoH/DoT**: Always use encrypted DNS
3. **Constant Rate**: Enable constant-rate traffic for high-security scenarios
4. **Regular Rotation**: Keep key rotation enabled (default)

## References

- [Traffic Analysis](https://en.wikipedia.org/wiki/Traffic_analysis)
- [DNS Privacy](https://www.ietf.org/archive/id/draft-ietf-dprive-rfc7626-bis-00.html)
- [Metadata Minimization](https://www.ietf.org/archive/id/draft-irtf-pearg-metadata-minimization-00.html)

