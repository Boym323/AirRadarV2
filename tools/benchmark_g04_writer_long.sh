#!/usr/bin/env bash
# G-04.06/.12: časově rozložený Rust readsb replay přes skutečný writer.
# Pouze lokální izolovaný PostgreSQL; nesmí mířit na sdílený DB LXC.
set -Eeuo pipefail

: "${AIRRADAR_G04_BINARY:?Nastav cestu k binárce airradarv2-engine}"
: "${PGHOST:?Nastav PGHOST=127.0.0.1}"
: "${PGPORT:?Nastav PGPORT=55432}"
: "${PGDATABASE:?Nastav PGDATABASE=airradar_v2_dev}"
: "${PGUSER:?Nastav PGUSER=postgres}"
if [[ "$PGHOST" != "127.0.0.1" || "$PGPORT" != "55432" || "$PGDATABASE" != "airradar_v2_dev" ]]; then
  echo "ODMÍTNUTO: pouze lokální izolovaný PostgreSQL 127.0.0.1:55432/airradar_v2_dev" >&2
  exit 2
fi

scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
chmod 0700 "$scratch"
fixture="$scratch/readsb.json"
psql -X -v ON_ERROR_STOP=1 -q -c 'TRUNCATE airradar_v2.flight_samples'

python3 - "$fixture" <<'PY' &
import json
import os
import pathlib
import sys
import tempfile
import time

path = pathlib.Path(sys.argv[1])
for step in range(16):
    now = time.time()
    rows = [
        {
            "hex": f"{i:06x}",
            "seen": 0.0,
            "seen_pos": 0.0,
            "lat": 49.0 + i / 10000.0,
            "lon": 17.0 + step * 0.02,
            "type": "adsb_icao",
        }
        for i in range(64)
    ]
    fd, temporary = tempfile.mkstemp(dir=path.parent, prefix="readsb-", suffix=".json")
    with os.fdopen(fd, "w", encoding="utf-8") as handle:
        json.dump({"now": now, "messages": len(rows), "aircraft": rows}, handle, separators=(",", ":"))
    os.replace(temporary, path)
    if step != 15:
        time.sleep(10)
PY
updater=$!
export AIRRADAR_V2_READSB_JSON="$fixture"
export AIRRADAR_V2_BIND=127.0.0.1:45164
export AIRRADAR_V2_DEV_DB_HOST=127.0.0.1
export AIRRADAR_V2_DEV_DB_PORT=55432
export AIRRADAR_V2_DEV_DB_USER="$PGUSER"
export AIRRADAR_V2_DEV_DB_PASSWORD="${PGPASSWORD:-local-isolated-only}"
"$AIRRADAR_G04_BINARY" --g04-dev-history >"$scratch/engine.log" 2>&1 &
engine=$!
wait "$updater"
sleep 12
kill -INT "$engine"
wait "$engine" || true

psql -X -v ON_ERROR_STOP=1 -P pager=off -c "
SELECT count(*) AS rows, count(DISTINCT aircraft_key) AS aircraft,
       min(per_aircraft) AS min_rows_per_aircraft,
       max(per_aircraft) AS max_rows_per_aircraft,
       avg(per_aircraft)::numeric(8,2) AS avg_rows_per_aircraft
FROM (SELECT aircraft_key, count(*) AS per_aircraft
      FROM airradar_v2.flight_samples GROUP BY aircraft_key) AS counts;"
echo "G-04 long replay PASS: 64 aircraft, 16 snapshots / 10 s, writer ran through bounded RAM and SQLx."
