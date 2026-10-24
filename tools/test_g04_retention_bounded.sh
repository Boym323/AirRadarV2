#!/usr/bin/env bash
# G-04.09/J09: výhradně izolovaný PostgreSQL CI (nikoli sdílený DB LXC).
set -Eeuo pipefail
[[ "$PGDATABASE" == "airradar_v2_dev" && "$PGUSER" == "postgres" ]] || exit 78

psql -X -q -v ON_ERROR_STOP=1 <<'SQL'
CREATE ROLE airradar_v2_dev_maintenance
  LOGIN PASSWORD 'ci-only-temporary-retention-test'
  NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION;
GRANT CONNECT ON DATABASE airradar_v2_dev TO airradar_v2_dev_maintenance;
GRANT USAGE ON SCHEMA airradar_v2 TO airradar_v2_dev_maintenance;
GRANT SELECT, DELETE ON airradar_v2.flight_samples TO airradar_v2_dev_maintenance;
INSERT INTO airradar_v2.flight_samples
  (bucket_at, aircraft_key, position_at, latitude, longitude)
VALUES
  (date_bin('30 seconds', now() - interval '32 days', '1970-01-01 UTC'::timestamptz),
   'icao:abc999', now() - interval '32 days', 49.2, 17.6),
  (date_bin('30 seconds', now() - interval '1 day', '1970-01-01 UTC'::timestamptz),
   'icao:abc998', now() - interval '1 day', 49.2, 17.6)
ON CONFLICT DO NOTHING;
SQL
old="$(psql -X -tA -v ON_ERROR_STOP=1 -c "SELECT count(*) FROM airradar_v2.flight_samples WHERE aircraft_key='icao:abc999'")"
[[ "$old" == 1 ]] || exit 1

set +e
env PGDATABASE=other_db PGUSER=airradar_v2_dev_maintenance \
  AIRRADAR_G04_RETENTION_APPROVED=YES \
  bash database/maintenance/g04_retention_bounded.sh >/dev/null 2>&1
wrong_database=$?
env PGDATABASE=airradar_v2_dev PGUSER=airradar_v2_dev_maintenance \
  AIRRADAR_G04_RETENTION_APPROVED=NO \
  bash database/maintenance/g04_retention_bounded.sh >/dev/null 2>&1
missing_approval=$?
set -e
[[ "$wrong_database" == 78 && "$missing_approval" == 78 ]] || exit 1

env PGDATABASE=airradar_v2_dev PGUSER=airradar_v2_dev_maintenance \
  PGPASSWORD=ci-only-temporary-retention-test \
  AIRRADAR_G04_RETENTION_APPROVED=YES \
  bash database/maintenance/g04_retention_bounded.sh

old="$(psql -X -tA -v ON_ERROR_STOP=1 -c "SELECT count(*) FROM airradar_v2.flight_samples WHERE aircraft_key='icao:abc999'")"
recent="$(psql -X -tA -v ON_ERROR_STOP=1 -c "SELECT count(*) FROM airradar_v2.flight_samples WHERE aircraft_key='icao:abc998'")"
[[ "$old" == 0 && "$recent" == 1 ]] || {
  echo "G-04 retention: mazani stareho vs. zachovani noveho nesedi" >&2
  exit 1
}
echo "PASS G-04.09: samostatná role, odmítnutí špatné DB/GO, retence pouze V2"
