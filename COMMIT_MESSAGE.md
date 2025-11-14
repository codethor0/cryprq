# Commit Message for Web-Only Refactor

```
Refactor: Web-only CrypRQ with Dockerized deploy

## Summary
Ruthlessly de-scoped CrypRQ to focus exclusively on web experience.
Removed all non-web platforms, simplified repo structure, and made it
Docker-deployable.

## What Was Removed
- Platform-specific code: android/, apple/, macos/, windows/, mobile/, gui/
- Build systems: Nix (flake.nix, shell.nix), Makefile, cbindgen.toml
- GitHub workflows: 14 workflows for mobile/GUI/release/QA removed
- Legacy code: Old reports, logs, QA directories, release artifacts
- Total: 420+ files removed

## What Was Kept
- Core web stack: web/ (Vite + React + TS), web/server/ (Node.js)
- Core Rust crates: crypto/, p2p/, node/, cli/, core/
- Docker support: Dockerfile, docker-compose.yml, docker-compose.vpn.yml
- Documentation: docs/OPERATOR_LOGS.md, docs/DOCKER_VPN_LOGS.md
- CI/CD: Simplified workflows (ci.yml, security-*, codeql.yml, web-preview.yml)

## New Files
- Dockerfile.web: Multi-stage Dockerfile for web stack
- docker-compose.web.yml: Web-focused Docker Compose
- WEB_ONLY_CHANGES.md: Detailed change log
- REFACTOR_PLAN.md: Refactoring plan document

## Changes Made
- Cargo.toml: Kept workspace members, added comments
- .github/workflows/ci.yml: Removed iOS/Android icon validation
- README.md: Updated for web-first architecture, added Docker quickstart
- OPERATOR_CHEAT_SHEET.txt: Updated with web-only commands

## Architecture
Frontend: Vite + React + TypeScript
Backend: Node.js Express (spawns Rust cryprq binary)
Core: Rust crates (crypto, p2p, node, cli)

## Recovery
Pre-refactor state preserved in tag: pre-web-split-20251113
To recover: git checkout pre-web-split-20251113

## Testing
- ✅ cargo check --workspace passes
- ✅ cargo test --lib --all passes
- ✅ npm run build (web) passes
- ⏳ Docker build (needs testing)

## Next Steps
1. Test: docker compose -f docker-compose.web.yml up --build
2. Verify: Web UI accessible at http://localhost:5173
3. Merge: When Docker deploy verified
```

