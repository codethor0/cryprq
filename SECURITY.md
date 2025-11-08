# Security Policy

## Supported Versions
- `main`: actively maintained.
- Future tagged releases: latest patch only.

No other branches receive security fixes.

## Reporting a Vulnerability
- Email: `codethor@gmail.com`
- Optional PGP encryption (replace with production key before publishing):
  ```
  -----BEGIN PGP PUBLIC KEY BLOCK-----
  Version: GnuPG v2

  mQENBFbj0hABCAC5Pqg0l4COREPLACEWITHREALKEYEXAMPLE==
  -----END PGP PUBLIC KEY BLOCK-----
  ```

### Disclosure Process
1. Acknowledge receipt within **72 hours**.
2. Provide triage status and remediation plan within **7 days**.
3. Coordinate fix release and public advisory with the reporter.

### Safe Harbor
We welcome good-faith research. Activities that follow this policy and avoid user data exfiltration or service disruption will not trigger legal action.

## Security Practices
- CI enforces `cargo fmt`, `cargo clippy`, `cargo test`, `cargo audit`, `cargo deny`, and CodeQL.
- Vendored `third_party/if-watch` reduces supply-chain drift.
- Reproducible build instructions are documented in `REPRODUCIBLE.md`.

## Contact Preferences
- Language: English.
- Include affected commit/tag, reproduction steps, and impact assessment.
- Encrypt sensitive details with the PGP key above when possible.

---

**Checklist**
- [ ] Report sent to `codethor@gmail.com`.
- [ ] (Optional) Payload encrypted with PGP.
- [ ] Disclosure timeline agreed with maintainers.
