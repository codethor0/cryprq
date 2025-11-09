#![no_main]

use libfuzzer_sys::fuzz_target;
use multiaddr::Multiaddr;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(addr) = s.parse::<Multiaddr>() {
            let canonical = addr.to_string();
            let _ = canonical.parse::<Multiaddr>();
        }
    }
});
