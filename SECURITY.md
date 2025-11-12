# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   |  |
| < 1.0   |                 |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security vulnerability, please follow these steps:

### 1. **Do NOT** open a public issue
Security vulnerabilities should be reported privately to prevent exploitation.

### 2. Email Security Team
Send an email to: **codethor@gmail.com**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### 3. Response Timeline
- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity (see below)

### 4. Severity Levels

#### Critical
- Remote code execution
- Authentication bypass
- Cryptographic weaknesses
- **Response**: Immediate (24-48 hours)

#### High
- Information disclosure
- Denial of service
- Privilege escalation
- **Response**: Within 7 days

#### Medium
- Local information disclosure
- Limited DoS
- **Response**: Within 30 days

#### Low
- Minor information leaks
- Best practice violations
- **Response**: Next release cycle

## Security Best Practices

### For Users
-  Keep CrypRQ updated to the latest version
-  Enable post-quantum encryption (default)
-  Use explicit peer allowlists
-  Review logs regularly
-  Report suspicious activity

### For Developers
-  Follow secure coding practices
-  Use `cargo audit` and `cargo deny`
-  Run fuzz tests before commits
-  Review cryptographic implementations carefully
-  Never commit secrets or keys

## Bug Bounty Program

**Status**: Coming Soon

We plan to launch a bug bounty program in the future. Check back for updates.

### Scope (Planned)
- Core cryptographic implementations
- Network protocol handling
- Authentication and authorization
- Key management and rotation

### Out of Scope (Planned)
- Social engineering attacks
- Physical access attacks
- Denial of service (unless critical)
- Issues in dependencies (report upstream)

## Security Updates

Security updates are released as soon as possible after a vulnerability is confirmed and fixed. Critical vulnerabilities may result in immediate releases.

## Disclosure Policy

We follow responsible disclosure:
1. Vulnerability reported privately
2. Issue confirmed and fix developed
3. Fix tested and deployed
4. Public disclosure (with credit to reporter)

## Contact

- **Security Email**: codethor@gmail.com
- **PGP Key**: [Coming Soon]
- **Security Advisories**: [GitHub Security Advisories](https://github.com/codethor0/cryprq/security/advisories)

## Acknowledgments

We thank security researchers who responsibly disclose vulnerabilities. Contributors will be credited (with permission) in security advisories.

---

**Last Updated**: 2025-11-11
