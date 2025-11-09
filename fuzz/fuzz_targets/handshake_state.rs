#![no_main]

use libfuzzer_sys::fuzz_target;
use once_cell::sync::Lazy;
use snow::{params::NoiseParams, Builder};

static NOISE_PARAMS: Lazy<NoiseParams> = Lazy::new(|| {
    "Noise_XX_25519_ChaChaPoly_BLAKE2s".
        parse()
        .expect("valid noise params")
});
static STATIC_KEY: [u8; 32] = [0u8; 32];

fuzz_target!(|data: &[u8]| {
    let builder = Builder::new(NOISE_PARAMS.clone()).local_private_key(&STATIC_KEY);
    if let Ok(mut responder) = builder.build_responder() {
        let mut buffer = vec![0u8; data.len() + 512];
        let _ = responder.read_message(data, &mut buffer);
    }
});
