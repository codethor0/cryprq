# Dependency Monitoring Schedule

## Weekly Dependency Checks

Run every Monday at 10 AM:

```bash
# Add to crontab (crontab -e)
0 10 * * 1 cd /path/to/cryprq && ./scripts/weekly-dependency-check.sh >> logs/deps.log 2>&1
```

Or run manually:
```bash
./scripts/weekly-dependency-check.sh
```

## What Gets Checked
1. Security vulnerabilities (cargo audit)
2. Outdated dependencies (cargo-outdated)
3. Dependency count and statistics

## Reports Location
- Security audit: `dependency-reports/audit-YYYYMMDD.log`
- Outdated deps: `dependency-reports/outdated-YYYYMMDD.log`

## Action Items
- Review reports weekly
- Update critical security vulnerabilities immediately
- Plan dependency updates monthly
- Test updates in development before production
