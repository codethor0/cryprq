use cryprq_crypto::make_kyber_keys;

fn main() {
    let (pk, _sk) = make_kyber_keys();
    println!("CrypRQ v0.0.1 – Kyber pk: {:02x}{:02x}…", pk[0], pk[1]);
}
