# Contributing to CrypRQ

Thank you for your interest in contributing! Please follow these guidelines to help us maintain a high-quality project.

## How to Contribute
- Fork the repository and create your branch from `main`.
- Sign all commits using SSH when possible (see SECURITY.md).
- Add SPDX headers to all new Rust source files.
- Write clear, descriptive commit messages.
- Run `cargo test --release` and `cargo clippy --all-targets --all-features`.
- Run `cargo audit` (optional, but strongly encouraged).
- Document any manual verification steps you performed in the pull request description.

## Pull Request Process
- Open a pull request against the `main` branch.
- Fill out the PR template and describe your changes, including which manual checks you ran.
- Link any related issues in your PR description.
- Respond to review comments and update your PR as needed.

## Code of Conduct
All contributors must follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## License
By contributing, you agree that your code may be licensed under the Apache 2.0 or MIT license.

---
SPDX-License-Identifier: Apache-2.0 OR MIT