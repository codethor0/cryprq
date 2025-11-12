# Performance Benchmarking Schedule

## Weekly Benchmarking

Run every Monday at 9 AM:

```bash
# Add to crontab (crontab -e)
0 9 * * 1 cd /path/to/cryprq && ./scripts/scheduled-benchmark.sh >> logs/benchmark.log 2>&1
```

Or run manually:
```bash
./scripts/scheduled-benchmark.sh
```

## Monthly Performance Review

First Monday of each month:
1. Review all weekly benchmarks
2. Compare with previous month
3. Identify regressions
4. Document findings

## Benchmark Metrics Tracked
- Binary size
- Startup time
- Build time
- Test execution time
- Memory usage (if valgrind available)

## Reports Location
- Weekly: `benchmarks/benchmark-YYYYMMDD.log`
- Comparison: `benchmarks/last-benchmark.log`
