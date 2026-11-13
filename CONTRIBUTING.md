# Contributing

CrypRQ is a research prototype. Contributions are welcome when they are small
enough to review and come with evidence that they work.

## Pull requests

- One topic per pull request, against `main`, from a short-lived branch.
- Describe what changes, why, and how it was tested. State what was not
  tested.
- Keep unrelated formatting, renames and dependency updates out of the change.
- Update the documentation that the change makes inaccurate, in the same pull
  request. Do not add claims the tests do not support.

## Tests

Run the checks that cover the change and report the actual result:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --locked
```

The end-to-end scripts, the container TUN test and the browser portal tests
are listed in the [README](README.md#validation-and-ci) and in
[CI gates](docs/CI_VPN_GATES.md). A test added for a fix should fail without
the fix. Tests must be reproducible on a clean checkout: no dependence on local
state, fixed ports that may be taken, or the host's network configuration.

Do not weaken, skip or delete a failing check to make a result green, and do
not add advisory exceptions without a documented reason.

## Security-sensitive changes

Changes to the handshake, key derivation, the record layer, replay protection,
peer pinning, identity storage, file publication or the portal's request
boundary need more than passing tests:

- explain the security property affected and why it still holds;
- keep wire formats and negotiation unchanged, or state the versioning
  decision explicitly;
- add negative tests (wrong identity, tampering, replay, malformed input) for
  the new behaviour;
- never replace production cryptography with a test-only path.

Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md),
not through a pull request.

## History and signatures

The history of `main` consists of signed commits and is not rewritten. Do not
force push to shared branches, and do not rebase or squash commits that are
already on `main`. Sign your own commits with your own key and identity; do not
set another person's name or address as author or committer.

## License

Contributions are accepted under the [MIT License](LICENSE) of the project.
The logo is not covered by that license; see the README.
