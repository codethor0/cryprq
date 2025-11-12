.PHONY: help ops validate quick-smoke local-validate cleanup sanity observability one-shot ship github-sync

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

