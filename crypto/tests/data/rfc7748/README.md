# RFC 7748 X25519 Test Vectors

Official test vectors from RFC 7748 for X25519 ECDH.

## Download Instructions

Test vectors are in RFC 7748 Section 5.2:
https://www.rfc-editor.org/rfc/rfc7748.html#section-5.2

## Format

Test vectors include:
- Scalar (private key, 32 bytes)
- Input u-coordinate (public key, 32 bytes)
- Output u-coordinate (shared secret, 32 bytes)

## Usage

These vectors verify:
- X25519 scalar multiplication
- Shared secret derivation
- Known private/public pairs
- Edge cases (all-zero, all-one inputs)

