// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

//! ML-KEM-768 known-answer tests against the NIST ACVP vectors in
//! `kat_vectors/ml-kem-768-acvp.txt`. Every vector in the file is run; a
//! missing or short file fails the tests.

use cryprq_crypto::mlkem::{
    decapsulate, encapsulate_with_randomness, keypair_from_seed, Ciphertext, MlKemError, PublicKey,
    SecretKey, CIPHERTEXT_LEN, PUBLIC_KEY_LEN, SECRET_KEY_LEN,
};

const VECTORS: &str = include_str!("kat_vectors/ml-kem-768-acvp.txt");

fn hex(text: &str) -> Vec<u8> {
    assert!(text.len() % 2 == 0, "odd-length hex field");
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex digit"))
        .collect()
}

/// The fields of every vector of one kind, after its name and test-case id.
fn vectors(kind: &str, fields: usize, expected: usize) -> Vec<(String, Vec<String>)> {
    let found: Vec<_> = VECTORS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.split(' ').map(str::to_owned).collect::<Vec<_>>())
        .filter(|parts| parts[0] == kind)
        .map(|parts| {
            assert_eq!(parts.len(), fields + 2, "{kind} {}: field count", parts[1]);
            (parts[1].clone(), parts[2..].to_vec())
        })
        .collect();
    assert_eq!(found.len(), expected, "number of {kind} vectors");
    found
}

#[test]
fn key_generation_matches_the_nist_vectors() {
    for (id, fields) in vectors("keygen", 4, 25) {
        let (d, z, ek, dk) = (
            hex(&fields[0]),
            hex(&fields[1]),
            hex(&fields[2]),
            hex(&fields[3]),
        );
        let mut seed = [0u8; 64];
        seed[..32].copy_from_slice(&d);
        seed[32..].copy_from_slice(&z);
        let (public, secret) = keypair_from_seed(&seed);
        assert_eq!(public.as_bytes(), ek, "keygen {id}: encapsulation key");
        assert_eq!(secret.as_bytes(), dk, "keygen {id}: decapsulation key");
        assert_eq!(public.as_bytes().len(), PUBLIC_KEY_LEN);
        assert_eq!(secret.as_bytes().len(), SECRET_KEY_LEN);
        assert_eq!(
            secret.public_key(),
            public,
            "keygen {id}: derived public key"
        );
    }
}

#[test]
fn encapsulation_matches_the_nist_vectors() {
    for (id, fields) in vectors("encap", 4, 25) {
        let (ek, m, c, k) = (
            hex(&fields[0]),
            hex(&fields[1]),
            hex(&fields[2]),
            hex(&fields[3]),
        );
        let public = PublicKey::from_bytes(&ek).expect("valid encapsulation key");
        let randomness: [u8; 32] = m.as_slice().try_into().expect("32-byte m");
        let (secret, ciphertext) = encapsulate_with_randomness(&public, &randomness);
        assert_eq!(ciphertext.as_bytes(), c, "encap {id}: ciphertext");
        assert_eq!(secret.as_bytes(), k, "encap {id}: shared secret");
        assert_eq!(ciphertext.as_bytes().len(), CIPHERTEXT_LEN);
    }
}

#[test]
fn decapsulation_matches_the_nist_vectors_including_implicit_rejection() {
    let mut rejected = 0;
    for (id, fields) in vectors("decap", 4, 10) {
        let (dk, c, k) = (hex(&fields[0]), hex(&fields[1]), hex(&fields[2]));
        let secret_key = SecretKey::from_bytes(&dk).expect("valid decapsulation key");
        let ciphertext = Ciphertext::from_bytes(&c).expect("ciphertext length");
        // For a modified ciphertext the expected value is the implicit
        // rejection secret, not an error.
        assert_eq!(
            decapsulate(&ciphertext, &secret_key).as_bytes(),
            k,
            "decap {id}"
        );
        if fields[3] == "modified_ciphertext" {
            rejected += 1;
        }
    }
    assert!(
        rejected >= 3,
        "the vectors must include modified ciphertexts"
    );
}

#[test]
fn a_modified_ciphertext_yields_a_different_secret_than_the_sender_has() {
    let (id, fields) = vectors("encap", 4, 25).swap_remove(0);
    let (ek, m) = (hex(&fields[0]), hex(&fields[1]));
    let public = PublicKey::from_bytes(&ek).expect("valid encapsulation key");
    let randomness: [u8; 32] = m.as_slice().try_into().expect("32-byte m");
    let (sent, ciphertext) = encapsulate_with_randomness(&public, &randomness);

    // The matching decapsulation key is in the key-generation vectors only
    // for their own keys, so use a fresh pair for the round trip.
    let (public, secret_key) = keypair_from_seed(&[7u8; 64]);
    let (sent_here, ciphertext_here) = encapsulate_with_randomness(&public, &randomness);
    assert_eq!(
        decapsulate(&ciphertext_here, &secret_key),
        sent_here,
        "round trip"
    );

    for position in [0, CIPHERTEXT_LEN / 2, CIPHERTEXT_LEN - 1] {
        let mut bytes = ciphertext_here.as_bytes().to_vec();
        bytes[position] ^= 0x01;
        let modified = Ciphertext::from_bytes(&bytes).expect("ciphertext length");
        let first = decapsulate(&modified, &secret_key);
        assert_ne!(
            first, sent_here,
            "encap {id}: modified byte {position} accepted"
        );
        // Implicit rejection is deterministic for a given key and ciphertext.
        assert_eq!(decapsulate(&modified, &secret_key), first);
    }
    // A ciphertext made for another key is rejected the same way.
    assert_ne!(decapsulate(&ciphertext, &secret_key), sent);
}

#[test]
fn encapsulation_key_checks_match_the_nist_vectors() {
    let mut refused = 0;
    for (id, fields) in vectors("ekcheck", 3, 10) {
        let accepted = PublicKey::from_bytes(&hex(&fields[0])).is_ok();
        assert_eq!(accepted, fields[1] == "1", "ekcheck {id} ({})", fields[2]);
        refused += usize::from(!accepted);
    }
    assert!(refused >= 3, "the vectors must include invalid keys");
}

#[test]
fn decapsulation_key_checks_match_the_nist_vectors() {
    let mut refused = 0;
    for (id, fields) in vectors("dkcheck", 3, 10) {
        let accepted = SecretKey::from_bytes(&hex(&fields[0])).is_ok();
        assert_eq!(accepted, fields[1] == "1", "dkcheck {id} ({})", fields[2]);
        refused += usize::from(!accepted);
    }
    assert!(refused >= 3, "the vectors must include invalid keys");
}

#[test]
fn values_of_the_wrong_length_are_refused() {
    for len in [0, 1, PUBLIC_KEY_LEN - 1, PUBLIC_KEY_LEN + 1] {
        assert!(matches!(
            PublicKey::from_bytes(&vec![0u8; len]),
            Err(MlKemError::InvalidLength { expected: PUBLIC_KEY_LEN, actual }) if actual == len
        ));
    }
    for len in [0, SECRET_KEY_LEN - 1, SECRET_KEY_LEN + 1] {
        assert!(SecretKey::from_bytes(&vec![0u8; len]).is_err());
    }
    for len in [0, CIPHERTEXT_LEN - 1, CIPHERTEXT_LEN + 1] {
        assert!(Ciphertext::from_bytes(&vec![0u8; len]).is_err());
    }
    // Right length, but coefficients out of range.
    assert_eq!(
        PublicKey::from_bytes(&[0xffu8; PUBLIC_KEY_LEN]).err(),
        Some(MlKemError::InvalidKey)
    );
}
