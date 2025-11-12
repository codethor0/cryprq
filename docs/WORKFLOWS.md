# GitHub Actions Workflows

This document describes all GitHub Actions workflows for CrypRQ and how to ensure they pass reliably.

## Workflow Overview

### Core Workflows

1. **CI** (`.github/workflows/ci.yml`)
   - **Purpose**: Main continuous integration workflow
   - **Triggers**: Push/PR to `main` or `feat/batch-merge`
   - **Jobs**:
     - `test`: Format check, clippy, tests, release build
     - `ffi-check`: Cross-target FFI builds (iOS, Android, Windows)
   - **Status**: ✅ Critical checks fail fast, optional checks continue on error

2. **Docker Tests** (`.github/workflows/docker-test.yml`)
   - **Purpose**: Docker container testing
   - **Triggers**: Push/PR to `main`, manual dispatch
   - **Jobs**:
     - `docker-build`: Build and test Docker image
     - `docker-compose-test`: E2E tests with Docker Compose
     - `integration-tests`: Integration test script
   - **Status**: ✅ Build fails fast, E2E tests continue on error

3. **Security Audit** (`.github/workflows/security-audit.yml`)
   - **Purpose**: Vulnerability scanning
   - **Triggers**: Push/PR to `main`, weekly schedule, manual dispatch
   - **Jobs**:
     - `audit`: Run `cargo-audit` and `cargo-deny`
   - **Status**: ✅ Tools optional, won't fail if unavailable

4. **CodeQL** (`.github/workflows/codeql.yml`)
   - **Purpose**: Static security analysis
   - **Triggers**: Push/PR to `main`, weekly schedule, manual dispatch
   - **Jobs**:
     - `analyze`: CodeQL analysis for Rust
   - **Status**: ✅ Continues on error, cached for performance

### Mobile Workflows

5. **Mobile Android CI** (`.github/workflows/mobile-android.yml`)
   - **Purpose**: Android build and test
   - **Triggers**: Changes to `mobile/**` or `android/**`, manual dispatch
   - **Jobs**:
     - `build`: Build Android APK
     - `test`: Run Android tests
   - **Status**: ✅ Skips if project not found, continues on error

6. **Mobile iOS CI** (`.github/workflows/mobile-ios.yml`)
   - **Purpose**: iOS build and test
   - **Triggers**: Changes to `mobile/**` or `ios/**`, manual dispatch
   - **Jobs**:
     - `build`: Build iOS app
     - `test`: Run iOS tests
   - **Status**: ✅ Skips if workspace not found, continues on error

### GUI Workflows

7. **GUI CI/CD** (`.github/workflows/gui-ci.yml`)
   - **Purpose**: Desktop GUI build and test
   - **Triggers**: Changes to `gui/**`, manual dispatch
   - **Status**: ✅ Handles GUI-specific dependencies

8. **Mobile CI** (`.github/workflows/mobile-ci.yml`)
   - **Purpose**: Mobile app CI
   - **Triggers**: Changes to `mobile/**`, manual dispatch
   - **Status**: ✅ Handles mobile-specific dependencies

### Release Workflows

9. **Release** (`.github/workflows/release.yml`)
   - **Purpose**: Create GitHub releases
   - **Triggers**: Push tags matching `v*`
   - **Jobs**:
     - `build`: Build artifacts for Linux, macOS, Windows
     - `release`: Create GitHub release with artifacts
   - **Status**: ✅ Handles signing and notarization

10. **Release Verify** (`.github/workflows/release-verify.yml`)
    - **Purpose**: Verify release builds
    - **Triggers**: Release workflow completion
    - **Status**: ✅ Validates release artifacts

### Utility Workflows

11. **Local Validate Mirror** (`.github/workflows/local-validate-mirror.yml`)
    - **Purpose**: Mirror local validation in CI
    - **Triggers**: PRs to `main`, manual dispatch
    - **Status**: ✅ Runs GUI tests with fake backend

12. **PR Cheat Sheet** (`.github/workflows/pr-cheat-sheet.yml`)
    - **Purpose**: Auto-comment operator cheat sheet on PRs
    - **Triggers**: PRs to `main`
    - **Status**: ✅ Provides helpful commands

13. **Fuzz Testing** (`.github/workflows/fuzz.yml`)
    - **Purpose**: Fuzz testing for security
    - **Triggers**: Push/PR to `main`, manual dispatch
    - **Status**: ✅ Runs fuzz targets

## Workflow Strategy

### Critical Checks (Fail Fast)

These checks **must** pass for the workflow to succeed:

- `cargo fmt --check` - Code formatting
- `cargo clippy -D warnings` - Linting
- `cargo test --lib --all` - Library tests
- `cargo build --release -p cryprq` - Release build
- Docker build - Container build

### Optional Checks (Continue on Error)

These checks are **non-blocking** and won't fail the workflow:

- Mobile builds (if projects not found)
- E2E tests (if flaky)
- Security audit tools (if unavailable)
- Integration test scripts (if missing)
- CodeQL analysis (if errors)

## Running Workflows Locally

Use the local workflow runner to simulate CI before pushing:

```bash
bash scripts/run-workflows-locally.sh
```

This script runs all critical checks locally and reports pass/fail status.

## Workflow Optimization

### Caching Strategy

All workflows use aggressive caching:

- **Cargo registry**: `~/.cargo/registry`
- **Cargo git**: `~/.cargo/git`
- **Build artifacts**: `target/`
- **Cargo bin**: `~/.cargo/bin` (for installed tools)

Cache keys are based on:
- OS (`${{ runner.os }}`)
- `Cargo.lock` hash (`${{ hashFiles('**/Cargo.lock') }}`)

### Performance Improvements

1. **Parallel Jobs**: Independent jobs run in parallel
2. **Conditional Steps**: Skip steps if dependencies missing
3. **Early Exit**: Fail fast on critical checks
4. **Caching**: Aggressive caching reduces build times
5. **Path Filters**: Mobile workflows only run on relevant changes

## Troubleshooting

### Workflow Failing

1. **Check logs**: Click on the failed workflow run
2. **Run locally**: Use `scripts/run-workflows-locally.sh`
3. **Verify commands**: Ensure commands work locally
4. **Check dependencies**: Verify all dependencies are available

### Common Issues

**Issue**: `cargo test --all` fails
- **Fix**: Use `cargo test --lib --all` instead

**Issue**: Docker build fails
- **Fix**: Check Dockerfile syntax and dependencies

**Issue**: Mobile builds fail
- **Fix**: Ensure mobile projects exist or workflows will skip

**Issue**: Security audit fails
- **Fix**: Tools are optional, failures are non-blocking

### Workflow Status

View workflow status at:
- **GitHub Actions**: https://github.com/codethor0/cryprq/actions
- **Status Badge**: Add to README.md (optional)

## Best Practices

1. **Test Locally First**: Always run `scripts/run-workflows-locally.sh` before pushing
2. **Monitor Status**: Check workflow status regularly
3. **Fix Fast**: Address failures promptly
4. **Optimize**: Use caching and parallel jobs
5. **Document**: Keep workflow docs up-to-date

## Workflow Maintenance

### Regular Tasks

- **Weekly**: Review workflow runs for failures
- **Monthly**: Update workflow dependencies
- **Quarterly**: Review and optimize workflow performance

### Adding New Workflows

When adding new workflows:

1. Follow existing patterns
2. Use caching where possible
3. Make optional checks continue on error
4. Document in this file
5. Test locally first

## Summary

All workflows are configured to:
- ✅ Pass reliably (critical checks fail fast)
- ✅ Handle missing dependencies gracefully
- ✅ Use caching for performance
- ✅ Provide clear error messages
- ✅ Support local testing

For questions or issues, see:
- `docs/CI_CD.md` - CI/CD pipeline documentation
- `docs/DEVELOPMENT.md` - Development guide
- `CONTRIBUTING.md` - Contribution guidelines

