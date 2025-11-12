# RFC 8439 ChaCha20-Poly1305 Test Vectors

Official test vectors from RFC 8439 for ChaCha20-Poly1305 AEAD.

## Download Instructions

Download from RFC 8439 Appendix A.5:
https://www.rfc-editor.org/rfc/rfc8439.html#appendix-A.5

Or use the test vectors provided in the RFC document.

## Format

Test vectors include:
- Key (32 bytes)
- Nonce (12 bytes)
- Additional Data (variable)
- Plaintext (variable)
- Ciphertext (variable)
- Tag (16 bytes)

## Usage

These vectors verify:
- Encryption correctness
- Decryption correctness
- AEAD tag verification
- Nonce handling
- Additional data handling
- Tamper detection

