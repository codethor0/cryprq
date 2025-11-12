# FIPS 203 ML-KEM (Kyber768) KAT Vectors

Official Known-Answer Test vectors from NIST FIPS 203.

## Download Instructions

Download from NIST CSRC:
https://csrc.nist.gov/projects/post-quantum-cryptography/post-quantum-cryptography-standardization/round-3-submissions

For ML-KEM (Kyber768), download:
- `PQCkemKAT_2400.rsp` (Kyber768 test vectors)

Place the file in this directory.

## Format

The vectors follow the NIST KAT format:
```
count = 0
seed = ...
pk = ...
sk = ...
ct = ...
ss = ...
```

## Usage

The KAT loader (`crypto/tests/kat_loader.rs`) will parse these vectors and verify:
- Key generation
- Encapsulation
- Decapsulation
- Shared secret matching
- Negative cases (wrong key, tampered ciphertext)

