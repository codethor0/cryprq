use blake3::Hasher;
use bs58::encode;
use zeroize::Zeroize;
use std::fmt;

#[derive(Clone, Eq, PartialEq)]
pub struct AnonId([u8; 6]);

impl AnonId {
    pub fn from_pk(pk: &[u8]) -> Self {
        let hash = Hasher::new().update(pk).finalize();
        let mut id = [0u8; 6];
        id.copy_from_slice(&hash.as_bytes()[..6]);
        Self(id)
    }
}

impl fmt::Display for AnonId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", encode(self.0).into_string())
    }
}

impl Drop for AnonId {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn same_pk_same_anonid() {
        let pk = [42u8; 32];
        let a = AnonId::from_pk(&pk);
        let b = AnonId::from_pk(&pk);
        assert_eq!(a, b);
    }
    #[test]
    fn diff_pk_diff_anonid() {
        let pk1 = [1u8; 32];
        let pk2 = [2u8; 32];
        let a = AnonId::from_pk(&pk1);
        let b = AnonId::from_pk(&pk2);
        assert_ne!(a, b);
    }
    #[test]
    fn display_base58() {
        let pk = [99u8; 32];
        let id = AnonId::from_pk(&pk);
        let s = format!("{}", id);
        assert!(s.len() > 6);
    }
}
