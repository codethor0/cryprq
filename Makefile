.PHONY: help icons icons-verify icons-verify-fast icons-validate-ios icons-validate-android icons-rebuild docs-no-emoji docs-lint docs-links web-preview mac-app qa-web-mac ops validate quick-smoke local-validate cleanup sanity observability one-shot ship github-sync

help:
	@echo "CrypRQ Operations"
	@echo "================="
	@echo ""
	@echo "Most-used commands:"
	@echo "  make ops              - Show operator cheat sheet"
	@echo "  make validate         - Quick smoke test"
	@echo "  make one-shot         - Full workflow (validate + PR)"
	@echo "  make ship             - Full workflow with ship"
	@echo "  make github-sync      - GitHub sync + CI verification + optional ship"
	@echo ""
	@echo "Validation:"
	@echo "  make quick-smoke      - Fast local sanity (30s)"
	@echo "  make local-validate   - Full validation"
	@echo ""
	@echo "Monitoring:"
	@echo "  make sanity           - Pre-release sanity checks"
	@echo "  make observability    - Post-release monitoring"
	@echo ""
	@echo "Utilities:"
	@echo "  make cleanup          - Clean Docker + artifacts"
	@echo ""
	@echo "Icons:"
	@echo "  make icons            - Generate icons for all platforms"
	@echo "  make icons-verify-fast - Fast preflight gate"
	@echo "  make icons-verify    - Full verification with reports"
	@echo "  make icons-validate-ios - Validate iOS AppIcon Contents.json"
	@echo "  make icons-validate-android - Validate Android mipmap densities"
	@echo "  make icons-rebuild   - Generate -> verify -> rebuild"
	@echo ""
	@echo "Documentation:"
	@echo "  make docs-no-emoji   - Fail if any emoji/shortcodes remain"
	@echo "  make docs-lint       - Markdownlint over repo"
	@echo "  make docs-links      - Check links in all Markdown files"
	@echo ""
	@echo "Platform Builds:"
	@echo "  make web-preview     - Run web cross-browser Playwright tests"
	@echo "  make mac-app         - Package macOS .app bundle"
	@echo "  make qa-web-mac      - Run web QA + macOS packaging (with quality gates)"
	@echo ""
	@echo "See OPERATOR_CHEAT_SHEET.txt for complete reference."

ops:
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "🧭 Operator Cheat Sheet"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@if [ -f "OPERATOR_CHEAT_SHEET.txt" ]; then \
		cat OPERATOR_CHEAT_SHEET.txt; \
	else \
		echo "⚠️  OPERATOR_CHEAT_SHEET.txt not found"; \
	fi

validate: quick-smoke

quick-smoke:
	@./scripts/quick-smoke.sh

local-validate:
	@./scripts/local-validate.sh

cleanup:
	@./scripts/cleanup.sh

sanity:
	@./scripts/sanity-checks.sh

observability:
	@./scripts/observability-checks.sh

one-shot:
	@./scripts/one-shot.sh

ship:
	@SHIP=true RUN_POST=true ./scripts/one-shot.sh --ship --post

github-sync:
	@./scripts/github-sync.sh


icons: ## Generate icons for all platforms
	bash scripts/generate-icons.sh

icons-verify-fast: ## Fast preflight gate for icon coverage
	bash scripts/verify-icons-min.sh

icons-verify: ## Full verification with reports
	bash scripts/verify-icons.sh

icons-validate-ios: ## Validate iOS AppIcon Contents.json
	bash scripts/ios-contents-validate.sh

icons-validate-android: ## Validate Android mipmap densities
	bash scripts/android-mipmap-validate.sh

icons-rebuild: ## Generate -> verify -> rebuild packages
	bash scripts/rebuild-with-icons.sh

docs-no-emoji: ## Fail if any emoji/shortcodes remain
	bash scripts/no-emoji-gate.sh

docs-lint: ## Markdownlint over repo
	npx --yes markdownlint-cli2 "**/*.md" "!**/node_modules/**" "!**/target/**" "!**/dist/**" "!**/build/**"

docs-links: ## Check links in all Markdown files
	bash scripts/check-doc-links.sh

web-preview: ## Run web cross-browser Playwright tests
	npm run --silent test:web:install
	npm run --silent test:web

mac-app: ## Package macOS .app bundle
	bash scripts/package-mac-app.sh

qa-web-mac: ## Run web QA + macOS packaging with quality gates
	bash scripts/qa-web-and-mac.sh
