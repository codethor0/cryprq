# Getting Started with CrypRQ

## Quick Start

### 1. Install CrypRQ

**Desktop:**
- Download from [GitHub Releases](https://github.com/codethor0/cryprq/releases)
- Install the package for your platform (DMG on macOS, MSIX on Windows, AppImage/Deb on Linux)

**Mobile:**
- Android: Install from Google Play Store (when available)
- iOS: Install from App Store (when available)

### 2. First Run

1. Launch CrypRQ
2. Review and accept the Privacy Policy and EULA
3. Configure your first peer connection

### 3. Connect to a Peer

**Desktop:**
1. Go to **Peers** tab
2. Click **"Add Peer"**
3. Enter peer multiaddr (e.g., `/ip4/192.168.1.100/udp/9999/quic-v1/p2p/12D3KooW...`)
4. Click **"Connect"**

**Mobile:**
1. Go to **Peers** tab
2. Tap **"+"** button
3. Enter peer details
4. Tap **"Connect"**

## Understanding Post-Quantum Encryption

CrypRQ uses **ML-KEM (Kyber768) + X25519 hybrid encryption** by default. This provides:

- ✅ Protection against future quantum computer attacks
- ✅ Defense-in-depth security
- ✅ Automatic key rotation every 5 minutes

**Recommendation**: Keep post-quantum encryption enabled. See [Post-Quantum Encryption Guide](./post-quantum.md) for details.

## Key Settings

### Rotation Interval
- **Default**: 5 minutes (300 seconds)
- **Range**: 1 minute to 1 hour
- **Recommendation**: Keep default for optimal security

### Post-Quantum Encryption
- **Default**: Enabled
- **Recommendation**: Keep enabled

### Log Level
- **Options**: Error, Warn, Info, Debug
- **Default**: Info
- **Debug**: Use for troubleshooting

## Troubleshooting

### Connection Issues

1. **Check peer allowlist**: Ensure peer ID is in allowlist (if configured)
2. **Verify network**: Ensure UDP port is accessible
3. **Check logs**: View logs in **Logs** tab or modal
4. **Test reachability**: Use "Test Reachability" button in Peers view

### Performance Issues

1. **Disable charts**: If charts cause performance issues, disable via feature flags
2. **Reduce log level**: Set to "Error" or "Warn" for better performance
3. **Check metrics**: View throughput and latency in Dashboard

## Next Steps

- [Post-Quantum Encryption Guide](./post-quantum.md)
- [Security Model](../security.md)
- [Configuration Guide](../configuration.md)
- [FAQ](../faq.md)

