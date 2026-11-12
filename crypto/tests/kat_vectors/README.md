# ML-KEM-768 known-answer vectors

`ml-kem-768-acvp.txt` holds the ML-KEM-768 test cases of the NIST ACVP server
(`usnistgov/ACVP-Server`, `gen-val/json-files/ML-KEM-keyGen-FIPS203` and
`ML-KEM-encapDecap-FIPS203`): 25 key generations, 25 encapsulations,
10 decapsulations (five with a modified ciphertext, which must yield the
implicit-rejection secret), 10 encapsulation-key checks and 10
decapsulation-key checks. `crypto/tests/kat_loader.rs` runs every line.
