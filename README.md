# CrypRQ  
*Post-quantum, zero-trust, ransom-timer VPN that burns every key after 5 min.*

[![CI](https://github.com/cryprq/cryprq/actions/workflows/ci.yml/badge.svg)](https://github.com/cryprq/cryprq/actions)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue)](LICENSE-GPL)

---

## ⚡  Status  
Pre-alpha – looking for crypto & p2p reviewers.  
**First 10 merged PRs** receive gen-0 zk-bandwidth tokens.

---

## 🧪  Build
```bash
cargo build --release -p cryprq
./target/release/cryprq --help
cat > Cargo.toml <<'EOF'
[workspace]
members = ["crypto","p2p","node","cli"]
resolver = "2"

[workspace.package]
version = "0.0.1"
authors = ["CrypRQ Contributors"]
edition = "2021"
license = "GPL-3.0"
repository = "https://github.com/cryprq/cryprq"
rust-version = "1.75"

[profile.release]
lto = "thin"
strip = true
codegen-units = 1
