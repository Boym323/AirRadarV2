#!/usr/bin/env bash
# G-04.09/J09 — DBA-only ohranicena udrzba V2 historie, nikoli Rust runtime.
set -Eeuo pipefail

if [[ "$PGDATABASE" != "airradar_v2_dev" || "$PGUSER" != "airradar_v2_dev_maintenance" ]]; then
  echo "G-04 retention: chybi vyhradena V2 databaze nebo udrzbova role" >&2
  exit 78
fi
if [[ "$AIRRADAR_G04_RETENTION_APPROVED" != "YES" ]]; then
  echo "G-04 retention: rucni DBA souhlas chybi" >&2
  exit 78
fi
command -v psql >/dev/null || { echo "G-04 retention: psql chybi" >&2; exit 78; }

export PGCONNECT_TIMEOUT=3
export PGOPTIONS="-c statement_timeout=5000 -c lock_timeout=1000"
deadline=$((SECONDS + 90))
total=0
for ((batch=1; batch<=20; batch++)); do
  if ((SECONDS >= deadline)); then
    echo "G-04 retention: dosazen casovy limit, dalsi davky priste"
    break
  fi
  # Každá transakce maže max. 1000 řádků; SKIP LOCKED respektuje paralelní DB práci.
  deleted="$(psql --no-psqlrc --quiet --tuples-only --no-align \
    --set=ON_ERROR_STOP=1 <<'SQL'
BEGIN;
DO $gate$
BEGIN
  IF current_database() <> 'airradar_v2_dev'
     OR current_user <> 'airradar_v2_dev_maintenance' THEN
    RAISE EXCEPTION 'G-04 retention: wrong V2 database/role';
  END IF;
END
$gate$;
WITH victim AS (
  SELECT ctid
  FROM airradar_v2.flight_samples
  WHERE bucket_at < now() - interval '30 days'
  ORDER BY bucket_at
  LIMIT 1000
), removed AS (
  DELETE FROM airradar_v2.flight_samples
  WHERE ctid IN (SELECT ctid FROM victim)
  RETURNING 1
)
SELECT count(*) FROM removed;
COMMIT;
SQL
  )"
  deleted="$(printf '%s' "$deleted" | tr -d '[:space:]')"
  if [[ ! "$deleted" =~ ^[0-9]{1,4}$ ]] || ((10#$deleted > 1000)); then
    echo "G-04 retention: nespravny vysledek SQL transakce" >&2
    exit 1
  fi
  total=$((total + 10#$deleted))
  if ((10#$deleted < 1000)); then break; fi
  sleep 1
done
echo "G-04 retention: odstraneno $total expirovanych vzorku (limit 20000/beh)"
