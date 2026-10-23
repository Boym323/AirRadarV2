#!/usr/bin/env bash
# G-04.12: skutečný Rust -> RAM sampler -> bounded batch -> SQLx -> PostgreSQL.
# Spouštět pouze proti lokálnímu izolovanému PostgreSQL, nikdy proti DB LXC.
set -Eeuo pipefail

: "${AIRRADAR_G04_BINARY:?Nastav cestu k binárce airradarv2-engine}"
: "${PGHOST:?Nastav PGHOST=127.0.0.1}"
: "${PGPORT:?Nastav PGPORT=55432}"
: "${PGDATABASE:?Nastav PGDATABASE=airradar_v2_dev}"
: "${PGUSER:?Nastav PGUSER=postgres}"

if [[ "$PGHOST" != "127.0.0.1" || "$PGPORT" != "55432" || "$PGDATABASE" != "airradar_v2_dev" ]]; then
  echo "ODMÍTNUTO: benchmark smí použít pouze lokální izolovaný PostgreSQL 127.0.0.1:55432/airradar_v2_dev" >&2
  exit 2
fi

scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
chmod 0700 "$scratch"

for profile in 1 2 4; do
  fixture="$scratch/fixture-${profile}x.json"
  input_count=$((profile * 1250))
  expected=$((input_count < 2048 ? input_count : 2048))
  port=$((45100 + profile))
  python3 - "$fixture" "$input_count" <<'PY'
import json
import pathlib
import sys
import time

path = pathlib.Path(sys.argv[1])
count = int(sys.argv[2])
now = time.time()
rows = [
    {
        "hex": f"{i:06x}",
        "seen": 0.0,
        "seen_pos": 0.0,
        "lat": 48.0 + (i % 100) / 1000.0,
        "lon": 16.0 + (i % 100) / 1000.0,
        "type": "adsb_icao",
    }
    for i in range(count)
]
path.write_text(json.dumps({"now": now, "messages": count, "aircraft": rows}, separators=(",", ":")), encoding="utf-8")
PY
  psql -X -v ON_ERROR_STOP=1 -q -c 'TRUNCATE airradar_v2.flight_samples'
  psql -X -v ON_ERROR_STOP=1 -q -c "SELECT pg_stat_reset_shared('wal')"
  wal_before="$(psql -X -At -v ON_ERROR_STOP=1 -c 'SELECT wal_bytes FROM pg_stat_wal')"
  export AIRRADAR_V2_READSB_JSON="$fixture"
  export AIRRADAR_V2_BIND="127.0.0.1:${port}"
  export AIRRADAR_V2_DEV_DB_HOST=127.0.0.1
  export AIRRADAR_V2_DEV_DB_PORT=55432
  export AIRRADAR_V2_DEV_DB_USER="$PGUSER"
  export AIRRADAR_V2_DEV_DB_PASSWORD="${PGPASSWORD:-}"
  "$AIRRADAR_G04_BINARY" --g04-dev-history >"$scratch/engine-${profile}x.log" 2>&1 &
  pid=$!
  start_ns="$(date +%s%N)"
  curl -sN --max-time 90 "http://127.0.0.1:${port}/api/v1/stream" >"$scratch/sse-${profile}x.dat" 2>/dev/null &
  sse_pid=$!
  for _ in $(seq 1 100); do
    rows="$(psql -X -At -v ON_ERROR_STOP=1 -c 'SELECT count(*) FROM airradar_v2.flight_samples')"
    [[ "$rows" -ge "$expected" ]] && break
    kill -0 "$pid" 2>/dev/null || { cat "$scratch/engine-${profile}x.log" >&2; exit 1; }
    sleep 1
  done
  rows="$(psql -X -At -v ON_ERROR_STOP=1 -c 'SELECT count(*) FROM airradar_v2.flight_samples')"
  [[ "$rows" -ge "$expected" ]]
  end_ns="$(date +%s%N)"
  utime="$(awk '{print $14}' "/proc/${pid}/stat")"
  stime="$(awk '{print $15}' "/proc/${pid}/stat")"
  rss_kib="$(awk '/VmRSS:/ {print $2}' "/proc/${pid}/status")"
  kill -INT "$pid"
  wait "$pid" || true
  kill "$sse_pid" 2>/dev/null || true
  wal_after="$(psql -X -At -v ON_ERROR_STOP=1 -c 'SELECT wal_bytes FROM pg_stat_wal')"
  sse_bytes="$(wc -c <"$scratch/sse-${profile}x.dat")"
  elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
  wal_bytes=$((wal_after - wal_before))
  dropped=$((input_count - rows))
  echo "G-04.12 writer ${profile}x: input=${input_count} accepted=${rows} dropped_by_bound=${dropped} elapsed_ms=${elapsed_ms} batches_ceiling=$(( (expected + 127) / 128 )) cpu_ticks=$((utime + stime)) rss_kib=${rss_kib} wal_bytes=${wal_bytes} sse_bytes=${sse_bytes}"
done
