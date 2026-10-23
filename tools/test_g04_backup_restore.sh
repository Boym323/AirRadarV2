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
psql -X -v ON_ERROR_STOP=1 -q -d postgres -c "DROP DATABASE airradar_v2_restore_ci WITH (FORCE)"
echo "G-04.11 CI restore PASS: původní i obnovené počty shodné ($original)"
