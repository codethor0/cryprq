# CI Maintenance Guide

## Overview
This document outlines how to keep CI green and maintain high code quality standards.

## CI Workflow Structure

### Main Jobs
1. **Build and Test** (`test` job)
   - Format check (`cargo fmt`)
   - Clippy linting (`cargo clippy`)
   - Unit tests (`cargo test`)
   - Release build (`cargo build --release`)
   - **This job MUST pass for CI to be green**

2. **FFI Cross-Target Builds** (`ffi-check` job)
   - Optional cross-compilation checks
   - Non-blocking (won't fail CI)
   - Requires Android NDK toolchains (may not be available in CI)

## Keeping CI Green

### Before Pushing
Run the CI health check locally:
```bash
./scripts/ci-health-check.sh
```

This runs all the same checks as CI:
- Format check
- Clippy linting
- Build
- Tests

### Pre-commit Hook
A pre-commit hook automatically formats Rust code before commits:
- Located at `.git/hooks/pre-commit-format`
- Automatically runs `cargo fmt --all` on `.rs` files
- Ensures code is always formatted before commit

### Common Issues

#### Format Failures
```bash
# Fix formatting
cargo fmt --all
git add -u
git commit --amend --no-edit
```

#### Clippy Warnings
```bash
# See warnings
cargo clippy --all-targets --all-features -- -D warnings

# Auto-fix some issues
cargo clippy --fix --all-targets --all-features
```

#### Build Failures
- Check for compilation errors
- Ensure all dependencies are in `Cargo.toml`
- Run `cargo update` if needed

#### Test Failures
- Run tests locally: `cargo test --lib --all`
- Fix failing tests before pushing

## Workflow Configuration

### Required Jobs
Only the `test` job is required for CI to pass. The `ffi-check` job is optional and won't fail the workflow.

### Non-Blocking Steps
Several steps are marked as `continue-on-error: true`:
- Icon generation (requires ImageMagick)
- Icon verification (optional)
- Cryptographic validation (optional script)
- Secret scanning (optional script)
- Exploratory testing (optional script)
- Performance benchmarks (optional script)

These steps provide additional checks but won't fail CI if they're unavailable or fail.

## Branch Protection

For public repositories, consider enabling branch protection rules:
1. Require status checks to pass before merging
2. Require the "Build and Test" job to pass
3. Require branches to be up to date before merging
4. Require pull request reviews

## Monitoring

Check CI status:
- GitHub Actions: https://github.com/codethor0/cryprq/actions
- Badge: Shows latest run status on main branch
- Email notifications: Configured in repository settings

## Troubleshooting

### CI Failing Locally But Passing Remotely
- Ensure Rust version matches (1.83.0)
- Clear cargo cache: `cargo clean`
- Update dependencies: `cargo update`

### CI Passing Locally But Failing Remotely
- Check for environment-specific issues
- Review CI logs for specific errors
- Ensure all files are committed

### FFI Build Failures
- These are non-blocking and won't fail CI
- FFI builds require platform-specific toolchains
- Android builds require NDK (not available in CI)
- iOS builds require Xcode (macOS runners only)
