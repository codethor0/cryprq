# Security Checks and Technology Validation

This document describes the comprehensive security checks and technology validation processes for CrypRQ.

## Security Checks

### Secret Scanning

**Purpose**: Detect hardcoded secrets and sensitive information in the codebase.

**Script**: `scripts/secret-scan.sh`

**Usage**:
```bash
bash scripts/secret-scan.sh
```

**What it checks**:
- Hardcoded passwords
- API keys
- Private keys
- Tokens (GitHub, Slack, etc.)
- Bearer tokens
- Certificate private keys

**Exclusions**:
- Test files
- Example files
- Placeholder values
- Type names (e.g., `PrivateKey`, `SecretKey`)

**Integration**: Runs automatically in CI and security-checks workflow.

### Dependency Scanning

**Purpose**: Scan dependencies for known vulnerabilities.

**Tools**:
- `cargo audit` - Checks for known vulnerabilities
- `cargo deny` - Checks for advisories, bans, licenses

**Usage**:
```bash
cargo audit --deny warnings
cargo deny check
```

**Integration**: Runs automatically in security-checks workflow.

### Static Analysis

**Purpose**: Detect security vulnerabilities through static code analysis.

**Tool**: CodeQL

**Script**: `scripts/codeql-analysis.sh`

**Usage**:
```bash
bash scripts/codeql-analysis.sh
```

**Integration**: Runs automatically in CodeQL workflow (`.github/workflows/codeql.yml`).

### Dynamic Analysis

**Purpose**: Test the application in runtime conditions.

**Script**: `scripts/dynamic-analysis.sh`

**Usage**:
```bash
bash scripts/dynamic-analysis.sh
```

**What it checks**:
- Help command functionality
- Invalid argument handling
- Memory leaks (if Valgrind available)
- Fuzzing readiness

**Integration**: Runs automatically in security-checks workflow.

### Security Audits

**Purpose**: Regular security audits to ensure application remains secure.

**Script**: `scripts/security-audit.sh`

**Usage**:
```bash
bash scripts/security-audit.sh
```

**What it checks**:
- `cargo audit` for vulnerabilities
- `cargo deny` for advisories
- Unsafe code usage
- Hardcoded secrets

**Integration**: Runs automatically in security-audit workflow.

## Technology Validation

### Cryptographic Algorithm Validation

**Purpose**: Ensure all cryptographic algorithms function correctly.

**Script**: `scripts/crypto-validation.sh`

**Usage**:
```bash
bash scripts/crypto-validation.sh
```

**What it validates**:
- ML-KEM (Kyber768) + X25519 hybrid handshake
- Post-Quantum Pre-Shared Keys (PPKs)
- PQC Suite (ML-KEM768, ML-KEM1024, X25519)
- Zero-Knowledge Proofs framework
- Key rotation and expiration

**Integration**: Runs automatically in CI and security-checks workflows.

### End-to-End Testing

**Purpose**: Validate the complete application flow.

**Script**: `scripts/end-to-end-tests.sh`

**Usage**:
```bash
bash scripts/end-to-end-tests.sh
```

**What it tests**:
- Docker Compose services startup
- Listener readiness
- Dialer connectivity
- Integration test scripts

**Integration**: Runs automatically in security-checks workflow.

### Performance Testing

**Purpose**: Ensure application meets performance benchmarks.

**Script**: `scripts/performance-tests.sh`

**Usage**:
```bash
bash scripts/performance-tests.sh
```

**What it tests**:
- Cryptographic operation benchmarks
- Network throughput
- CPU and memory usage

**Integration**: Can be run manually or in CI.

### Docker Environment Validation

**Purpose**: Validate application in Docker environment.

**Usage**:
```bash
# Start services
docker compose up -d cryprq-listener

# Run tests
docker exec -it cryprq-listener bash -c "cargo test"

# Validate crypto
docker exec -it cryprq-listener bash -c "cargo run --example crypto_tests"
```

## Workflow Integration

### CI Workflow

The main CI workflow (`.github/workflows/ci.yml`) includes:
- Cryptographic validation
- Secret scanning

### Security Checks Workflow

The security-checks workflow (`.github/workflows/security-checks.yml`) includes:
- Secret scanning
- Dependency scanning
- Cryptographic validation
- Dynamic analysis
- End-to-end tests

### CodeQL Workflow

The CodeQL workflow (`.github/workflows/codeql.yml`) includes:
- Static security analysis
- Automated vulnerability detection

## Continuous Monitoring

### GitHub Actions

All security checks run automatically on:
- Push to `main` branch
- Pull requests
- Weekly schedule (for some checks)
- Manual dispatch

### Viewing Results

- **GitHub Actions**: https://github.com/codethor0/cryprq/actions
- **CodeQL Results**: https://github.com/codethor0/cryprq/security/code-scanning
- **Dependency Alerts**: https://github.com/codethor0/cryprq/security/dependabot

## Best Practices

1. **Run locally first**: Always run security checks locally before pushing
2. **Fix immediately**: Address security issues as soon as they're detected
3. **Regular audits**: Run security audits regularly (weekly schedule)
4. **Monitor alerts**: Check GitHub security alerts regularly
5. **Update dependencies**: Keep dependencies up-to-date

## Troubleshooting

### Secret Scan False Positives

If secret scan reports false positives:
1. Check if it's in a test/example file
2. Verify it's a placeholder value
3. Add exclusion pattern if needed

### Dependency Vulnerabilities

If vulnerabilities are found:
1. Check if fix is available (`cargo update`)
2. Review vulnerability severity
3. Update or replace vulnerable dependency
4. Test thoroughly after update

### CodeQL False Positives

If CodeQL reports false positives:
1. Review the alert in GitHub Security tab
2. Mark as false positive if appropriate
3. Add code comments to explain why it's safe

## Summary

All security checks and technology validation processes are:
- ✅ Automated in CI/CD
- ✅ Documented and scripted
- ✅ Integrated into workflows
- ✅ Regularly scheduled
- ✅ Available for local testing

For questions or issues, see:
- `docs/SECURITY.md` - Security policy
- `docs/WORKFLOWS.md` - Workflow documentation
- `CONTRIBUTING.md` - Contribution guidelines

