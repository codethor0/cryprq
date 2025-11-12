# Documentation Guardrails

This document describes the automated guardrails in place to keep documentation production-grade and emoji-free.

## Guardrails

### 1. No-Emoji Gate

**Script**: `scripts/no-emoji-gate.sh`

**Purpose**: Fail fast if any Markdown file contains Unicode emojis or ``

**Usage**:
```bash
bash scripts/no-emoji-gate.sh
# or
make docs-no-emoji
```

**Integration**:
- Pre-commit hook: Runs automatically on commits touching `.md` files
- CI: Runs in `.github/workflows/docs-ci.yml` on PRs and pushes

### 2. Link Checker

**Script**: `scripts/check-doc-links.sh`

**Purpose**: Validate internal and external links in all Markdown files

**Usage**:
```bash
bash scripts/check-doc-links.sh
# or
make docs-links
```

**Tools**:
- Primary: `markdown-link-check` (Node.js, via npx)
- Fallback: `lychee` (Rust)

**Configuration**: `.mlc.json` - Retry settings and ignore patterns

**Integration**:
- CI: Runs in `.github/workflows/docs-ci.yml`

### 3. Markdownlint

**Purpose**: Enforce consistent Markdown formatting

**Usage**:
```bash
npx --yes markdownlint-cli2 "**/*.md" "!**/node_modules/**"
# or
make docs-lint
```

**Configuration**: `.markdownlint.json`
- Line length: 120 characters
- HTML allowed (MD033: false)
- First heading level not enforced (MD041: false)

**Integration**:
- CI: Runs in `.github/workflows/docs-ci.yml`

### 4. Prettier (Non-Blocking)

**Purpose**: Format Markdown files consistently

**Usage**:
```bash
npx --yes prettier -c "**/*.md"
```

**Integration**:
- CI: Runs in `.github/workflows/docs-ci.yml` (non-blocking, warnings only)

## CI Workflow

**Workflow**: `.github/workflows/docs-ci.yml`

**Triggers**:
- Pull requests touching Markdown files or guardrail scripts
- Pushes to `main` branch touching Markdown files

**Steps**:
1. No-emoji gate (fails if emojis found)
2. Prettier format check (non-blocking)
3. Markdownlint (fails on lint errors)
4. Link check (fails on broken links)

## Pre-Commit Hook

**Location**: `.git/hooks/pre-commit`

**Checks**:
- Icon verification (for icon-related files)
- No-emoji gate (for Markdown files)

**Bypass**: Use `git commit --no-verify` for automated commits (e.g., `clean-docs.sh`)

## Makefile Targets

```bash
make docs-no-emoji  # Fail if any emoji/shortcodes remain
make docs-lint      # Markdownlint over repo
make docs-links     # Check links in all Markdown files
```

## Standardized Structure

All README files should follow this order:

1. **Title** - Single H1
2. **Overview** - Brief description
3. **Quickstart** - Fastest path to running
4. **Installation/Prerequisites** - Requirements and setup
5. **Build/Run** - Local, Docker, CI instructions
6. **Configuration** - Settings and options
7. **Security/Support** - Security notes and support info
8. **Contributing** - How to contribute
9. **License** - License information

## Master Prompt (for AI agents)

Act as Documentation Gatekeeper. For every PR and for main:

1. Run `scripts/no-emoji-gate.sh` and fail if any Markdown contains Unicode emojis or ``.
2. Ensure README.md and all docs/**/*.md follow the standardized section order: Title, Overview, Quickstart, Installation/Prerequisites, Build/Run (local/Docker/CI), Configuration, Security/Support, Contributing, License.
3. Run Prettier and markdownlint; fix simple issues (line length, headings, lists, tables).
4. Run link checks with `scripts/check-doc-links.sh`; fix internal anchors and obvious 404s. For external rate limits, retry via config.
5. Update `docs/DOCS_CLEANUP_SUMMARY.md` with changed files and fixes.
6. If any step fails, post a concise comment with file:line and required change; block merge.
7. Never add emojis or decorative badges; keep only essential CI/license badges.

## Maintenance

To update guardrails:

1. Modify scripts in `scripts/`
2. Update configs (`.markdownlint.json`, `.mlc.json`)
3. Test locally: `make docs-no-emoji docs-lint docs-links`
4. Commit changes
5. CI will automatically enforce new rules

