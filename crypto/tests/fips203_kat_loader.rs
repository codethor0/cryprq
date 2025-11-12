// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

// FIPS 203 ML-KEM (Kyber768) KAT vector loader and parser

#[cfg(not(test))]
extern crate alloc;
#[cfg(not(test))]
use alloc::string::String;
#[cfg(not(test))]
use alloc::vec::Vec;

#[cfg(test)]
use std::string::String;
#[cfg(test)]
use std::vec::Vec;

/// FIPS 203 KAT vector structure
#[derive(Debug, Clone)]
pub struct Fips203KatVector {
    pub count: usize,
    pub seed: Vec<u8>,
    pub pk: Vec<u8>,
    pub sk: Vec<u8>,
    pub ct: Vec<u8>,
    pub ss: Vec<u8>,
}

/// Parse NIST PQC KAT format
/// Format:
/// count = <number>
/// seed = <hex>
/// pk = <hex>
/// sk = <hex>
/// ct = <hex>
/// ss = <hex>
pub fn parse_fips203_kat_file(contents: &str) -> Result<Vec<Fips203KatVector>, String> {
    let mut vectors = Vec::new();
    let mut current = Fips203KatVector {
        count: 0,
        seed: Vec::new(),
        pk: Vec::new(),
        sk: Vec::new(),
        ct: Vec::new(),
        ss: Vec::new(),
    };

    let mut in_vector = false;

    for line in contents.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key = value pairs
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "count" => {
                    // Save previous vector if exists
                    if in_vector {
                        vectors.push(current.clone());
                    }
                    current.count = value
                        .parse()
                        .map_err(|_| format!("Invalid count: {}", value))?;
                    in_vector = true;
                }
                "seed" => {
                    current.seed =
                        hex::decode(value).map_err(|e| format!("Invalid seed hex: {}", e))?;
                }
                "pk" => {
                    current.pk =
                        hex::decode(value).map_err(|e| format!("Invalid pk hex: {}", e))?;
                }
                "sk" => {
                    current.sk =
                        hex::decode(value).map_err(|e| format!("Invalid sk hex: {}", e))?;
                }
                "ct" => {
                    current.ct =
                        hex::decode(value).map_err(|e| format!("Invalid ct hex: {}", e))?;
                }
                "ss" => {
                    current.ss =
                        hex::decode(value).map_err(|e| format!("Invalid ss hex: {}", e))?;
                }
                _ => {
                    // Unknown key, skip
                }
            }
        }
    }

    // Save last vector
    if in_vector {
        vectors.push(current);
    }

    Ok(vectors)
}

/// Verify a FIPS 203 KAT vector
pub fn verify_fips203_vector(vector: &Fips203KatVector) -> Result<(), String> {
    use pqcrypto_mlkem::mlkem768::{decapsulate, encapsulate};
    use pqcrypto_traits::kem::{PublicKey, SecretKey};

    // Load public key from bytes
    let pk =
        PublicKey::from_bytes(&vector.pk).map_err(|e| format!("Invalid public key: {:?}", e))?;

    // Load secret key from bytes
    let sk =
        SecretKey::from_bytes(&vector.sk).map_err(|e| format!("Invalid secret key: {:?}", e))?;

    // Test encapsulation
    let (ss_encaps, ct) = encapsulate(&pk);

    // Verify ciphertext matches (if deterministic)
    // Note: ML-KEM encapsulation is randomized, so we verify decapsulation instead

    // Test decapsulation
    let ss_decaps = decapsulate(&ct, &sk);

    // Verify shared secrets match
    use pqcrypto_traits::kem::SharedSecret;
    if ss_encaps.as_bytes() != ss_decaps.as_bytes() {
        return Err("Encaps/decaps shared secrets don't match".to_string());
    }

    // Verify decapsulation with provided ciphertext
    use pqcrypto_traits::kem::Ciphertext;
    let ct_provided =
        Ciphertext::from_bytes(&vector.ct).map_err(|e| format!("Invalid ciphertext: {:?}", e))?;
    let ss_provided = decapsulate(&ct_provided, &sk);

    // Verify shared secret matches expected
    if ss_provided.as_bytes() != vector.ss {
        return Err("Decapsulation doesn't match expected shared secret".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kat_parser_structure() {
        // Test parser with minimal valid input
        let test_input = r#"
count = 0
seed = 000102030405060708090A0B0C0D0E0F
pk = 000102030405060708090A0B0C0D0E0F
sk = 000102030405060708090A0B0C0D0E0F
ct = 000102030405060708090A0B0C0D0E0F
ss = 000102030405060708090A0B0C0D0E0F
"#;

        let vectors = parse_fips203_kat_file(test_input).unwrap();
        assert_eq!(vectors.len(), 1);
        assert_eq!(vectors[0].count, 0);
    }
}
