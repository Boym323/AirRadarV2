#!/usr/bin/env bash
# G-04.11: izolovaný test pg_dump/pg_restore. NIKDY NEPOUŠTĚT na sdílené DB.
set -Eeuo pipefail
if [[ "${PGDATABASE:-}" != "airradar_v2_dev" || "${PGHOST:-}" != "127.0.0.1" ]]; then
  echo "ODMÍTNUTO: pouze izolovaná databáze GitHub CI" >&2
  exit 2
fi
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
chmod 0777 "$scratch"
psql -X -v ON_ERROR_STOP=1 -q -c "INSERT INTO airradar_v2.flight_samples
(bucket_at,aircraft_key,position_at,latitude,longitude)
VALUES ('2026-10-22 13:00:00+00','icao:789abc','2026-10-22 13:00:12+00',49.3,17.3)"
# Verze klienta pg_dump MUSÍ být nejméně 18; bez závislosti na Ubuntu psql 16.
docker run --rm --network host -e PGPASSWORD="${PGPASSWORD}" \
  -v "$scratch:/out" postgres:18 pg_dump -h 127.0.0.1 -U postgres \
  -d airradar_v2_dev -n airradar_v2 -Fc -f /out/g04.dump
test -s "$scratch/g04.dump"
psql -X -v ON_ERROR_STOP=1 -q -d postgres -c "CREATE DATABASE airradar_v2_restore_ci"
docker run --rm --network host -e PGPASSWORD="${PGPASSWORD}" \
  -v "$scratch:/out" postgres:18 pg_restore -h 127.0.0.1 -U postgres \
  -d airradar_v2_restore_ci --no-owner --no-privileges /out/g04.dump
original="$(psql -X -At -v ON_ERROR_STOP=1 -c 'SELECT count(*) FROM airradar_v2.flight_samples')"
restored="$(psql -X -At -v ON_ERROR_STOP=1 -d airradar_v2_restore_ci -c 'SELECT count(*) FROM airradar_v2.flight_samples')"
[[ "$original" = "$restored" && "$restored" -ge 1 ]]
# Retence nesmí být schopna smazat data jiné databáze (negativní test).
if psql -X -v ON_ERROR_STOP=1 -q -d airradar_v2_restore_ci \
  -f database/maintenance/g04_retention_once.sql >/dev/null 2>&1; then
  echo "FAIL: retence neodmítla cizí DB" >&2
  exit 1
fi
psql -X -v ON_ERROR_STOP=1 -q -d postgres -c "DROP DATABASE airradar_v2_restore_ci WITH (FORCE)"

# Jednorázový CI retenční test: 2 501 starých řádků se čistí po dávkách
# nejvýše 1 000, nový řádek zůstane a PK index zůstane použitelný.
psql -X -v ON_ERROR_STOP=1 -q <<'SQL'
INSERT INTO airradar_v2.flight_samples
  (bucket_at, aircraft_key, position_at, latitude, longitude)
SELECT date_bin('30 seconds', timestamptz '2026-01-01 00:00:00+00' + i * interval '30 seconds', timestamptz '1970-01-01 00:00:00+00'),
       'icao:' || lpad(to_hex(i), 6, '0'),
       timestamptz '2026-01-01 00:00:05+00' + i * interval '30 seconds',
       49.3, 17.3
FROM generate_series(1, 2501) AS g(i);
INSERT INTO airradar_v2.flight_samples
  (bucket_at, aircraft_key, position_at, latitude, longitude)
VALUES ('2099-01-01 00:00:00+00','icao:123abc','2099-01-01 00:00:05+00',49.3,17.3);
SQL
before_rows="$(psql -X -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM airradar_v2.flight_samples WHERE bucket_at < now() - interval '30 days'")"
[[ "$before_rows" = 2501 ]]
for expected in 1501 501 0; do
  psql -X -v ON_ERROR_STOP=1 -q -f database/maintenance/g04_retention_once.sql
  remaining="$(psql -X -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM airradar_v2.flight_samples WHERE bucket_at < now() - interval '30 days'")"
  [[ "$remaining" = "$expected" ]]
done
old="$(psql -X -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM airradar_v2.flight_samples WHERE aircraft_key LIKE 'icao:00000%'")"
new="$(psql -X -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM airradar_v2.flight_samples WHERE aircraft_key='icao:123abc'")"
index_count="$(psql -X -v ON_ERROR_STOP=1 -Atc "SELECT count(*) FROM pg_indexes WHERE schemaname='airradar_v2' AND tablename='flight_samples'")"
relation_bytes="$(psql -X -v ON_ERROR_STOP=1 -Atc "SELECT pg_total_relation_size('airradar_v2.flight_samples')")"
database_bytes="$(psql -X -v ON_ERROR_STOP=1 -Atc "SELECT pg_database_size(current_database())")"
[[ "$old" = 0 && "$new" = 1 && "$index_count" = 1 && "$relation_bytes" -gt 0 && "$database_bytes" -gt 0 ]]
echo "G-04.09/.11 CI PASS: restore=$original řádků, retenční dávky=1000/1000/501, staré=$old, nové=$new, indexy=$index_count, relation_bytes=$relation_bytes, database_bytes=$database_bytes"
