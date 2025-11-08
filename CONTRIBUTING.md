# Contributing to CrypRQ

We appreciate your time and expertise. Please follow the guidance below to streamline reviews and keep the project secure.

## Code of Conduct
All participation is bound by the [Code of Conduct](CODE_OF_CONDUCT.md).

## Environment Setup
```bash
rustup toolchain install 1.83.0
git clone https://github.com/codethor0/cryprq.git
cd cryprq
rustup override set 1.83.0
```

## Development Workflow
1. Create a topic branch from `main`.
2. Implement changes with tests and documentation updates.
3. Ensure commits are SSH-signed.
4. Run the required checks:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --release
   cargo audit --deny warnings
   cargo deny check advisories bans sources licenses --deny vulnerability --deny unmaintained --deny unsound --warn notice
   ./scripts/docker_vpn_test.sh
   ```
5. Update `README.md` and relevant `docs/` pages if behavior changes.
6. Submit a pull request using `.github/PULL_REQUEST_TEMPLATE.md`.

## Style & Quality
- Follow `rustfmt`; no unchecked warnings.
- Use `anyhow::Result` for CLI errors, `thiserror` for library errors.
- Prefer structured logging (`tracing`/`log` macros) over `println!`.
- Add SPDX/contact headers to new files via `scripts/add-headers.sh`.

## Testing
- Unit tests: `cargo test --release`.
- Integration: `./scripts/docker_vpn_test.sh` verifies listener/dialer.
- Security: `cargo audit` and `cargo deny` must pass before opening PRs.

## Documentation
- Docs site lives under `docs/` (MkDocs). Update pages relevant to your change.
- Keep README high-level; detailed procedures belong in docs.

## Communication
- Reference related issues in PR descriptions.
- Document manual verification steps in the PR checklist.
- Be responsive to review feedback and resolve discussions before merge.

---

**Checklist**
- [ ] Branch created from `main`.
- [ ] All lint/test/audit commands executed.
- [ ] Documentation updated.
- [ ] Commits signed and PR template completed.