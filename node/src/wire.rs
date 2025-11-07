pub const OUTER_LEN: usize = 1480;

pub fn pad_to_1480(plain: &[u8]) -> [u8; OUTER_LEN] {
    let mut buf = [0u8; OUTER_LEN];
    let len = plain.len().min(OUTER_LEN);
    buf[..len].copy_from_slice(&plain[..len]);
    // Pad with 0x00 (already zeroed)
    buf
}

pub fn unpad(buf: &[u8; OUTER_LEN]) -> Option<Vec<u8>> {
    let first_zero = buf.iter().position(|&b| b == 0x00).unwrap_or(OUTER_LEN);
    if first_zero == 0 { return None; }
    Some(buf[..first_zero].to_vec())
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;

    #[test]
    fn round_trip_random_slices() {
        let mut rng = rand::thread_rng();
        for len in 0..OUTER_LEN {
            let data: Vec<u8> = (0..len).map(|_| rng.gen_range(1..=255)).collect();
            let padded = pad_to_1480(&data);
            let unpadded = unpad(&padded).unwrap();
            assert_eq!(data, unpadded, "Failed at len={}", len);
        }
    }
}
