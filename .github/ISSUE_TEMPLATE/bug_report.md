---
name: Bug report
about: Help us diagnose and fix an issue in CrypRQ
title: "[Bug] "
labels: bug
assignees: ""
---

## Summary
What happened? Include expected vs actual behavior.

## Environment
- CrypRQ commit/tag:
- OS / Kernel:
- Rust version (`rustc --version`):
- Deployment method (bare metal, Docker, Nix):

## Steps to Reproduce
1. …
2. …
3. …

## Logs / Output
```
<paste relevant excerpts>
```

## Additional Context
Anything else we should know (network topology, configs, etc.).

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --release`
- [ ] `cargo audit --deny warnings`
- [ ] `cargo deny check advisories bans sources licenses --deny vulnerability --deny unmaintained --deny unsound --warn notice`

