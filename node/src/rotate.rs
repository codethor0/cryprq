// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

use blake3::Hasher;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::device_id::AnonId;
use zeroize::Zeroize;

pub fn keyburn_proof(old_pk: &[u8], anon_id: &AnonId) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time should be after UNIX_EPOCH")
        .as_secs() as u32;
    let mut input = Vec::with_capacity(old_pk.len() + 4);
    input.extend_from_slice(old_pk);
    input.extend_from_slice(&ts.to_le_bytes());
    let proof = Hasher::new().update(&input).finalize();
    format!(
        "cryprq_keyburn_proof{{anon_id=\"{}\"}} {:x}",
        anon_id,
        proof
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::time::{pause, advance, Duration};

    #[tokio::test]
    async fn keyburn_proof_changes_on_rotate() {
        pause();
        let pk = [42u8; 32];
        let anon = AnonId::from_pk(&pk);
        let proof1 = keyburn_proof(&pk, &anon);
        advance(Duration::from_secs(301)).await;
        let proof2 = keyburn_proof(&pk, &anon);
        assert_ne!(proof1, proof2);
    }
}
