#!/usr/bin/env bash
# G-04.04 / J09: destruktivní test POUZE pro krátkodobou izolovanou CI databázi.
set -Eeuo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
migration="$root/database/migrations/20261022210900_g04_initial"
if [[ "$PGDATABASE" != "airradar_v2_dev" || "$PGHOST" != "127.0.0.1" ]]; then
  echo "ODMÍTNUTO: migrační test smí běžet jen v izolované CI airradar_v2_dev na loopback" >&2
  exit 2
fi
sql() {
  psql -X -q -v ON_ERROR_STOP=1 -Atc "$1"
}
reject() {
  if psql -X -q -v ON_ERROR_STOP=1 -c "$1" >/dev/null 2>&1; then
    echo "FAIL: databáze přijala nepřípustnou operaci" >&2
    exit 1
  fi
}
reject_file() {
  if psql -X -q -v ON_ERROR_STOP=1 -f "$1" >/dev/null 2>&1; then
    echo "FAIL: databáze přijala nepřípustný migrační rollback" >&2
    exit 1
  fi
}
echo "G-04.04: migrace nahoru"
psql -X -q -v ON_ERROR_STOP=1 -f "$migration.up.sql"
[[ "$(sql "SELECT to_regclass('airradar_v2.flight_samples') IS NOT NULL")" == "t" ]]
[[ "$(sql "SELECT count(*) FROM pg_indexes WHERE schemaname='airradar_v2' AND tablename='flight_samples'")" == "1" ]]

# ICAO a non-ICAO jsou různé identity, duplicitní SQL replay nezdvojí pozici.
sql "INSERT INTO airradar_v2.flight_samples
 (bucket_at,aircraft_key,position_at,latitude,longitude,ground_speed_kt,track_deg,squawk)
 VALUES ('2026-10-22 12:00:00+00','icao:abcdef','2026-10-22 12:00:05+00',49.2,17.3,240,359,'7700')
 ON CONFLICT (bucket_at,aircraft_key) DO NOTHING" >/dev/null
sql "INSERT INTO airradar_v2.flight_samples
 (bucket_at,aircraft_key,position_at,latitude,longitude)
 VALUES ('2026-10-22 12:00:00+00','icao:abcdef','2026-10-22 12:00:06+00',49.3,17.4)
 ON CONFLICT (bucket_at,aircraft_key) DO NOTHING" >/dev/null
sql "INSERT INTO airradar_v2.flight_samples
 (bucket_at,aircraft_key,position_at,latitude,longitude)
 VALUES ('2026-10-22 12:00:00+00','nonicao:abcdef','2026-10-22 12:00:07+00',49.3,17.4)" >/dev/null
[[ "$(sql "SELECT count(*) FROM airradar_v2.flight_samples")" == "2" ]]
[[ "$(sql "SELECT latitude FROM airradar_v2.flight_samples WHERE aircraft_key='icao:abcdef'")" == "49.2" ]]

reject "INSERT INTO airradar_v2.flight_samples VALUES
 ('2026-10-22 12:00:00+00','icao:badxxx','2026-10-22 12:00:05+00',49.2,17.3,NULL,NULL,NULL,NULL,NULL)"
reject "INSERT INTO airradar_v2.flight_samples VALUES
 ('2026-10-22 12:00:00+00','icao:123456','2026-10-22 12:00:31+00',49.2,17.3,NULL,NULL,NULL,NULL,NULL)"
reject "INSERT INTO airradar_v2.flight_samples VALUES
 ('2026-10-22 12:00:00+00','icao:123456','2026-10-22 12:00:05+00',95,17.3,NULL,NULL,NULL,NULL,NULL)"
reject "INSERT INTO airradar_v2.flight_samples VALUES
 ('2026-10-22 12:00:00+00','icao:123456','2026-10-22 12:00:05+00',49.2,17.3,NULL,NULL,360,NULL,NULL)"

# Ověření minimálních práv runtime role: bez DDL, UPDATE, DELETE či TRUNCATE.
sql "CREATE ROLE g04_ci_runtime NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE" >/dev/null
sql "GRANT USAGE ON SCHEMA airradar_v2 TO g04_ci_runtime" >/dev/null
sql "GRANT SELECT, INSERT ON airradar_v2.flight_samples TO g04_ci_runtime" >/dev/null
[[ "$(sql "SELECT has_table_privilege('g04_ci_runtime','airradar_v2.flight_samples','SELECT')")" == "t" ]]
[[ "$(sql "SELECT has_table_privilege('g04_ci_runtime','airradar_v2.flight_samples','UPDATE')")" == "f" ]]
[[ "$(sql "SELECT has_table_privilege('g04_ci_runtime','airradar_v2.flight_samples','DELETE')")" == "f" ]]
reject "SET ROLE g04_ci_runtime; CREATE TABLE airradar_v2.unauthorized (id integer)"
reject "SET ROLE g04_ci_runtime; UPDATE airradar_v2.flight_samples SET latitude=1"

echo "G-04.04: EXPLAIN pouze informativní, není výkonový benchmark DEV LXC"
psql -X -q -v ON_ERROR_STOP=1 -c "EXPLAIN (ANALYZE, BUFFERS)
 SELECT position_at,latitude,longitude FROM airradar_v2.flight_samples
 WHERE bucket_at >= '2026-10-22 12:00:00+00'
   AND bucket_at < '2026-10-22 12:01:00+00'
 ORDER BY bucket_at"

# Destruktivní rollback je zakázaný i v CI; ověříme, že je odmítnut.
echo "G-04.04: destruktivní rollback musí být odmítnut"
reject_file "$migration.down.sql"
# Opakované spuštění up migrace je bezpečné a existující data zůstávají.
psql -X -q -v ON_ERROR_STOP=1 -f "$migration.up.sql"
[[ "$(sql "SELECT to_regclass('airradar_v2.flight_samples') IS NOT NULL")" == "t" ]]
[[ "$(sql "SELECT count(*) FROM airradar_v2.flight_samples")" == "2" ]]
echo "PASS: G-04.04 integrita, 30s bucket, dedup, práva, opakované up a bezpečný rollback"
