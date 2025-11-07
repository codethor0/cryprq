# Security Policy

## Cryptography & Security Features
- Post-quantum key exchange (Kyber768, X25519)
- Ed25519 peer authentication
- ChaCha20-Poly1305 AEAD encryption
- BLAKE3 key derivation
- 5-minute key rotation (ransom-timer)
- Replay attack protection (sliding window)
- Rate limiting and buffer pooling

## CI & Dependency Audits
- All dependencies are checked with `cargo-deny` and security audit tools
- CI runs on every PR and commit, including reproducible builds and musl targets
- Docker and Nix builds recommended for secure deployment

## Reporting Vulnerabilities
If you discover a security vulnerability in CrypRQ, please report it by emailing security@codethor.net or opening a private issue on GitHub. Do not disclose vulnerabilities publicly until they have been reviewed and addressed.

## Supported Versions
Only the latest stable release is supported for security updates. Older versions may not receive patches.

## Responsible Disclosure
We ask that you give us a reasonable time to respond and address the issue before public disclosure. We will coordinate with you to release a fix and acknowledge your contribution if desired.

## Contact
- Email: security@codethor.net
- GitHub Issues: https://github.com/codethor0/cryprq/issues

---
SPDX-License-Identifier: Apache-2.0 OR MIT