# Observability and Metrics Plan

## Success Metrics to Track

### 1. Pipeline Performance Metrics
- **Median pipeline time**: Target < 5 minutes
- **P95 pipeline time**: Target < 8 minutes
- **Queue wait time**: Target < 2 minutes
- **Cache hit rate**: Target > 70%

### 2. Reliability Metrics
- **Flake rate**: Target < 2%
- **Job success rate**: Target > 98%
- **Retry rate**: Track but minimize

### 3. Storage Metrics
- **Total disk usage**: Enforce < 10GB cap
- **Artifact storage**: Track per workflow
- **Cache storage**: Track per workflow
- **Log size**: Enforce < 5MB per log file

### 4. Efficiency Metrics
- **Unnecessary runs avoided**: Track via path filters
- **Concurrency cancellations**: Track duplicate run prevention
- **Storage reclaimed**: Daily cleanup reports

## Implementation Plan

### Phase 1: Baseline Measurement (Day 1-2)
1. Add metrics collection to existing workflows
2. Establish baseline measurements
3. Document current state

### Phase 2: Instrumentation (Day 3-4)
1. Add timing measurements to key jobs
2. Track cache hit/miss rates
3. Monitor disk usage before/after cleanup
4. Log queue wait times

### Phase 3: Reporting (Day 5-7)
1. Generate weekly metrics report
2. Compare before/after improvements
3. Identify optimization opportunities
4. Set 7-day improvement targets

## Metrics Collection Strategy

### GitHub Actions Implementation

```yaml
# Add to each workflow job
- name: Record job start time
  run: echo "JOB_START=$(date +%s)" >> $GITHUB_ENV

- name: Record job end time and calculate duration
  if: always()
  run: |
    JOB_END=$(date +%s)
    DURATION=$((JOB_END - ${{ env.JOB_START }}))
    echo "JOB_DURATION=$DURATION" >> $GITHUB_ENV
    echo "Job duration: ${DURATION}s"
```

### Cache Hit Rate Tracking

```yaml
- name: Cache cargo directories
  id: cache-cargo
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      ${{ runner.os }}-cargo-

- name: Record cache status
  run: |
    if [ "${{ steps.cache-cargo.outputs.cache-hit }}" == "true" ]; then
      echo "CACHE_HIT=true" >> $GITHUB_ENV
    else
      echo "CACHE_HIT=false" >> $GITHUB_ENV
    fi
```

### Disk Usage Tracking

```yaml
- name: Check disk usage
  run: |
    echo "=== Disk Usage Before ==="
    df -h
    echo ""
    echo "=== Workspace Usage ==="
    du -sh ${{ github.workspace }}/* 2>/dev/null | sort -h | tail -10
    echo ""
    DISK_USAGE=$(df -BG . | tail -1 | awk '{print $3}' | sed 's/G//')
    echo "DISK_USAGE_GB=$DISK_USAGE" >> $GITHUB_ENV
    if [ "$DISK_USAGE" -gt 10 ]; then
      echo "⚠️ Disk usage exceeds 10GB cap: ${DISK_USAGE}GB"
      exit 1
    fi
```

### Log Size Enforcement

```yaml
- name: Trim large logs
  run: |
    MAX_LOG_MB=5
    find . -type f -name "*.log" -size +${MAX_LOG_MB}M -exec bash -c '
      file="$1"
      size_mb=$(stat -c%s "$file" | awk "{print \$1/1024/1024}")
      echo "Trimming log: $file (${size_mb}MB)"
      tail -n 10000 "$file" > "${file}.tmp" && mv "${file}.tmp" "$file"
    ' _ {} \;
```

## Weekly Metrics Report Template

```markdown
# CI Metrics Report - Week of YYYY-MM-DD

## Performance Metrics
- **Median Pipeline Time**: X.X minutes (target: < 5 min)
- **P95 Pipeline Time**: X.X minutes (target: < 8 min)
- **Queue Wait Time**: X.X minutes (target: < 2 min)
- **Cache Hit Rate**: XX% (target: > 70%)

## Reliability Metrics
- **Flake Rate**: X.X% (target: < 2%)
- **Job Success Rate**: XX% (target: > 98%)
- **Total Runs**: XXX
- **Failed Runs**: XX

## Storage Metrics
- **Total Disk Usage**: X.X GB (cap: 10 GB)
- **Artifact Storage**: X.X GB
- **Cache Storage**: X.X GB
- **Storage Reclaimed This Week**: X.X GB

## Efficiency Metrics
- **Runs Avoided (Path Filters)**: XX
- **Duplicate Runs Cancelled**: XX
- **Estimated Time Saved**: X.X hours

## Improvements This Week
- [List specific improvements]

## Next Week Targets
- [Set specific targets]
```

## 7-Day Improvement Targets

### Baseline (Day 0)
- Median pipeline time: ~8 minutes
- Cache hit rate: ~40%
- Storage usage: ~10GB
- Flake rate: ~5%

### Target (Day 7)
- Median pipeline time: < 5 minutes (37% improvement)
- Cache hit rate: > 70% (75% improvement)
- Storage usage: < 5GB (50% reduction)
- Flake rate: < 2% (60% reduction)

## Automation

### Daily Metrics Collection
- Run as part of maintenance-cleanup.yml
- Collect metrics from GitHub Actions API
- Store in artifact for trend analysis

### Weekly Report Generation
- Generate markdown report
- Post as GitHub issue or PR comment
- Include charts/graphs if possible

## Tools and Integrations

### GitHub Actions Metrics
- Use GitHub Actions API to query run data
- Track workflow run durations
- Monitor cache usage

### External Monitoring (Optional)
- Datadog/New Relic integration
- Custom dashboard
- Alerting on thresholds

## Success Criteria

✅ Metrics collection automated
✅ Baseline established
✅ Weekly reports generated
✅ 7-day targets met
✅ Continuous improvement demonstrated
