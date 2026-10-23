#!/usr/bin/env bash
set -Eeuo pipefail

# G-04.11: pouze logický, nedestruktivní dump jediné V2 databáze.
# Heslo dodává PGPASSFILE/PGSERVICE mimo tento repozitář.
: "${AIRRADAR_V2_DEV_DB_HOST:?chybí host z chráněného prostředí}"
: "${AIRRADAR_V2_DEV_DB_USER:?chybí uživatel z chráněného prostředí}"
: "${AIRRADAR_V2_BACKUP_DIR:?chybí výstupní adresář}"
test "${AIRRADAR_V2_DEV_DB_NAME:-airradar_v2_dev}" = airradar_v2_dev
test "${AIRRADAR_V2_DEV_DB_PORT:-5432}" = 5432
test "${PGPASSFILE:-}" != ""  # heslo nesmí být v argumentu ani v repozitáři
test -r "$PGPASSFILE"
umask 077
install -d -m 0700 -- "$AIRRADAR_V2_BACKUP_DIR"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
out="$AIRRADAR_V2_BACKUP_DIR/airradar_v2_dev-$stamp.dump"
PGSSLMODE=disable pg_dump --format=custom --no-owner --no-privileges \
  --host="$AIRRADAR_V2_DEV_DB_HOST" --port="$AIRRADAR_V2_DEV_DB_PORT" \
  --username="$AIRRADAR_V2_DEV_DB_USER" --dbname=airradar_v2_dev --file="$out"
chmod 0600 -- "$out"
sha256sum -- "$out"
pg_restore --list -- "$out" >/dev/null
echo "PASS: pg_dump -Fc airradar_v2_dev; obnova se provádí pouze do nové izolované DB."
