# Deferred dependency updates

Routine Dependabot version-update pull requests are paused
(`open-pull-requests-limit: 0` in `.github/dependabot.yml`). The pause does not
change Dependabot alerts.

The updates below are known and deliberately not applied yet. None is rejected
for good.

## Cryptographic major versions: one coordinated, reviewed upgrade

These change the primitives used by the handshake and the record layer. They
must be upgraded together, with the known-answer tests, the handshake tests
and both end-to-end scripts run before and after. Do not apply them one by one.

| Crate | Locked | Available |
|-------|--------|-----------|
| chacha20poly1305 | 0.10.1 | 0.11.0 |
| hkdf | 0.12.4 | 0.13.0 |
| sha2 | 0.10.9 | 0.11.1 |
| x25519-dalek | 2.0.1 | 3.0.0 |

## Other Rust crates

| Crate | Locked | Available | Note |
|-------|--------|-----------|------|
| reqwest | 0.12.28 | 0.13.5 | Major update |
| clap | 4.5.53 | 4.5.60 | |
| blake3 | 1.8.2 | 1.8.7 | |
| futures | 0.3.31 | 0.3.34 | |
| libp2p components | core 0.43, swarm 0.47, quic 0.13 and others | the 0.57 release line | Changes swarm and QUIC/TLS APIs; to be done as one reviewed change |

## GitHub Actions major versions: one reviewed workflow change

| Action | Used | Available |
|--------|------|-----------|
| actions/setup-python | v5 | v7 |
| actions/setup-node | v4 | v7 |
| actions/github-script | v7 | v9 |
| actions/checkout | v4 | v7 |
| actions/cache | v4 | v6 |

## Re-enabling routine updates

Proposal, not active: grouped updates with one open pull request per
ecosystem, two routine update branches at most.

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 1
    groups:
      cryptography:
        patterns: ["chacha20poly1305", "hkdf", "sha2", "blake3", "x25519-dalek",
                   "ed25519-dalek", "pqcrypto-*", "rand", "rand_core", "zeroize"]
      rust-dependencies:
        patterns: ["*"]
        exclude-patterns: ["chacha20poly1305", "hkdf", "sha2", "blake3", "x25519-dalek",
                           "ed25519-dalek", "pqcrypto-*", "rand", "rand_core", "zeroize"]
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "monthly"
    open-pull-requests-limit: 1
    groups:
      github-actions:
        patterns: ["*"]
```

Order of work once enabled: the cryptography group as a single reviewed
change, then the general cargo group, then the GitHub Actions group.
