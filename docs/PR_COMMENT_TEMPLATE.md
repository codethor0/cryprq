# PR Comment Template

**Copy-paste this at the end of your PR:**

---

##  Operator Tip

**Golden path:** Connect → charts ≤3–5s → Rotate (toast ≤2s) → Disconnect

**Every 2h (first 24h):**
```bash
./scripts/observability-checks.sh
./scripts/sanity-checks.sh
```

**Rollback:** unmark desktop "Latest" release / pause mobile rollout.

---

