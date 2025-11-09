#!/usr/bin/env bash
set -euo pipefail

# Configuration
LOSSES=(0 10)
DELAYS=(0 50)
DURATION=${PERF_DURATION:-300} # total seconds for the mini-load
NETWORK="perf-net-$RANDOM"
IMAGE=${PERF_IMAGE:-cryprq-node:ci}
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/perf"

mkdir -p "$OUT_DIR/logs" "$OUT_DIR/tmp"
echo "loss,delay,attempts,success,success_within5s" > "$OUT_DIR/raw_results.csv"

cleanup() {
  if docker network inspect "$NETWORK" >/dev/null 2>&1; then
    docker ps -aq --filter "name=^perf-" | xargs -r docker rm -f >/dev/null 2>&1 || true
    docker network rm "$NETWORK" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

docker network create "$NETWORK" >/dev/null

combo_count=$(( ${#LOSSES[@]} * ${#DELAYS[@]} ))
per_combo=$(( DURATION / combo_count ))
if [ "$per_combo" -lt 45 ]; then
  per_combo=45
fi

for loss in "${LOSSES[@]}"; do
  for delay in "${DELAYS[@]}"; do
    listener="perf-listener-${loss}-${delay}"
    times_file="$OUT_DIR/tmp/times_loss${loss}_delay${delay}.txt"
    : > "$times_file"

    docker run -d --name "$listener" --network "$NETWORK" --cap-add=NET_ADMIN \
      "$IMAGE" --listen /ip4/0.0.0.0/udp/9999/quic-v1 >/dev/null

    sleep 5

    if [ "$loss" -ne 0 ] || [ "$delay" -ne 0 ]; then
      docker exec "$listener" sh -c "apt-get update >/dev/null && apt-get install -y iproute2 >/dev/null"
      tc_cmd="tc qdisc add dev eth0 root netem"
      if [ "$loss" -ne 0 ]; then
        tc_cmd+=" loss ${loss}%"
      fi
      if [ "$delay" -ne 0 ]; then
        tc_cmd+=" delay ${delay}ms"
      fi
      docker exec "$listener" sh -c "$tc_cmd"
    fi

    listener_ip=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$listener")
    if [ -z "$listener_ip" ]; then
      echo "Failed to determine listener IP for loss=$loss delay=$delay"
      exit 1
    fi

    attempts=0
    success=0
    success_within=0
    combo_end=$((SECONDS + per_combo))

    while [ "$SECONDS" -lt "$combo_end" ]; do
      attempts=$((attempts + 1))
      start_ns=$(date +%s%N)
      if docker run --rm --network "$NETWORK" "$IMAGE" --peer /ip4/${listener_ip}/udp/9999/quic-v1 >/dev/null 2>&1; then
        success=$((success + 1))
        end_ns=$(date +%s%N)
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        echo "$elapsed_ms" >> "$times_file"
        if [ "$elapsed_ms" -le 5000 ]; then
          success_within=$((success_within + 1))
        fi
      else
        echo "Handshake failed for loss=$loss delay=$delay" >&2
      fi
      sleep 0.3
    done

    docker logs "$listener" > "$OUT_DIR/logs/${listener}.log" 2>&1 || true
    docker rm -f "$listener" >/dev/null 2>&1 || true

    echo "${loss},${delay},${attempts},${success},${success_within}" >> "$OUT_DIR/raw_results.csv"
  done
done

# Idle CPU/RSS measurement with 3 looping dialers
idle_listener="perf-listener-idle"
docker run -d --name "$idle_listener" --network "$NETWORK" "$IMAGE" --listen /ip4/0.0.0.0/udp/9999/quic-v1 >/dev/null
sleep 5
idle_ip=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "$idle_listener")

for i in 1 2 3; do
  docker run -d --name "perf-idle-dialer-$i" --network "$NETWORK" --entrypoint /bin/sh \
    "$IMAGE" -c "while true; do /usr/local/bin/cryprq --peer /ip4/${idle_ip}/udp/9999/quic-v1; sleep 5; done" >/dev/null
done

sleep 20
idle_stats="$OUT_DIR/idle_stats.txt"
: > "$idle_stats"
for _ in $(seq 1 10); do
  docker stats --no-stream --format '{{.CPUPerc}},{{.MemUsage}}' "$idle_listener" >> "$idle_stats"
  sleep 5
done

docker logs "$idle_listener" > "$OUT_DIR/logs/${idle_listener}.log" 2>&1 || true
docker rm -f "$idle_listener" >/dev/null 2>&1 || true
docker ps -aq --filter "name=^perf-idle-dialer-" | xargs -r docker rm -f >/dev/null 2>&1 || true

cleanup
trap - EXIT

# Generate report and enforce thresholds
python3 - <<'PY'
import csv
import json
import statistics
from pathlib import Path
import re

root = Path(__file__).resolve().parent.parent
out_dir = root / "perf"
report = out_dir / "report.md"
failures_path = out_dir / "failures.json"
times_dir = out_dir / "tmp"

rows = []
with (out_dir / "raw_results.csv").open() as f:
    reader = csv.DictReader(f)
    for row in reader:
        times_file = times_dir / f"times_loss{row['loss']}_delay{row['delay']}.txt"
        times = []
        if times_file.exists():
            times = [int(line.strip()) for line in times_file.read_text().splitlines() if line.strip()]
        row["attempts"] = int(row["attempts"])
        row["success"] = int(row["success"])
        row["success_within"] = int(row["success_within5s"])
        row["times"] = times
        rows.append(row)

losses = sorted({int(r["loss"]) for r in rows})
delays = sorted({int(r["delay"]) for r in rows})

failures = []

with report.open("w") as f:
    f.write("# Performance Gate Summary\n\n")
    f.write("| Loss (%) | Delay (ms) | Attempts | Success | Success ≤5s | Success Rate | Median ms | Mean ms |\n")
    f.write("| --- | --- | --- | --- | --- | --- | --- | --- |\n")
    for row in rows:
        loss = int(row["loss"])
        delay = int(row["delay"])
        attempts = row["attempts"]
        success = row["success"]
        within = row["success_within"]
        rate = success / attempts if attempts else 0.0
        median = statistics.median(row["times"]) if row["times"] else None
        mean = statistics.mean(row["times"]) if row["times"] else None
        median_str = f"{median:.1f}" if median is not None else "—"
        mean_str = f"{mean:.1f}" if mean is not None else "—"
        f.write(f"| {loss} | {delay} | {attempts} | {success} | {within} | {rate:.3f} | {median_str} | {mean_str} |\n")
        if (loss <= 10 and delay <= 50):
            if rate < 0.99 or (median is not None and median > 400):
                failures.append({
                    "type": "handshake",
                    "loss": loss,
                    "delay": delay,
                    "success_rate": rate,
                    "median_ms": median,
                    "attempts": attempts,
                    "success": success,
                })

idle_stats_path = out_dir / "idle_stats.txt"
cpu_samples = []
rss_samples = []
if idle_stats_path.exists():
    pattern = re.compile(r"([0-9.]+)%?,\s*([0-9.]+)([A-Za-z]+)")
    for line in idle_stats_path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        m = pattern.match(line)
        if not m:
            continue
        cpu_val = float(m.group(1))
        mem_val = float(m.group(2))
        unit = m.group(3).lower()
        if unit.startswith("g"):
            mem_val *= 1024
        elif unit.startswith("k"):
            mem_val /= 1024
        cpu_samples.append(cpu_val)
        rss_samples.append(mem_val)
else:
    failures.append({"type": "idle_metrics", "reason": "missing idle stats"})

max_cpu = max(cpu_samples) if cpu_samples else None
max_rss = max(rss_samples) if rss_samples else None

with report.open("a") as f:
    f.write("\n## Idle Listener Metrics\n\n")
    f.write(f"- CPU samples (%): {cpu_samples}\n")
    f.write(f"- RSS samples (MiB): {rss_samples}\n")
    f.write(f"- Max CPU: {max_cpu if max_cpu is not None else 'n/a'}\n")
    f.write(f"- Max RSS: {max_rss if max_rss is not None else 'n/a'}\n")

if max_cpu is not None and max_cpu > 10.0:
    failures.append({"type": "idle_metrics", "metric": "cpu", "value": max_cpu})
if max_rss is not None and max_rss > 32.0:
    failures.append({"type": "idle_metrics", "metric": "rss", "value": max_rss})

if failures:
    failures_path.write_text(json.dumps(failures, indent=2))
else:
    if failures_path.exists():
        failures_path.unlink()
PY

if [ -f "$OUT_DIR/failures.json" ]; then
  echo "Performance gate failed. See perf/report.md for details."
  exit 1
fi

echo "Performance gate passed."

