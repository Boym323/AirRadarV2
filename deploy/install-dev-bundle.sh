#!/usr/bin/env bash
# Ruční atomická instalace nebo rollback G-00/G-01 artefaktu jen v DEV LXC.
set -Eeuo pipefail
if [ "$(id -u)" -ne 0 ] || [ "$#" -ne 1 ]; then
  echo "Použití: bash install-dev-bundle.sh /root/airradarv2-dev-linux-amd64-....tar.gz" >&2
  exit 2
fi
bundle="$(realpath -e "$1")"
if [ ! -f "$bundle.sha256" ]; then
  echo "ERROR: chybí soubor .sha256 z téhož GitHub Actions artefaktu" >&2
  exit 1
fi
if [ ! -f /etc/systemd/system/airradarv2-engine.service ] ||
   [ ! -L /etc/nginx/sites-enabled/airradarv2-dev ]; then
  echo "ERROR: nejdříve spusť bootstrap-dev-lxc.sh" >&2
  exit 1
fi
(
  cd "$(dirname "$bundle")"
  sha256sum -c "$(basename "$bundle").sha256"
)
tar -tzf "$bundle" >/tmp/airradarv2-dev-archive.list
if grep -E '(^/|(^|/)\.\.(/|$))' /tmp/airradarv2-dev-archive.list; then
  echo "ERROR: archiv obsahuje cestu mimo cílový adresář" >&2
  exit 1
fi

base=/opt/airradar-v2
work="$(mktemp -d "$base/releases/.incoming.XXXXXXXX")"
old=""
if [ -L "$base/current" ]; then
  old="$(readlink -f "$base/current")"
fi
activated=0

rollback() {
  exit_code="$1"
  trap - ERR
  echo "ERROR: instalace nebo health check selhal; vracím předchozí verzi." >&2
  if [ "$activated" -eq 1 ]; then
    if [ -n "$old" ] && [ -d "$old" ]; then
      ln -sfn "$old" "$base/.rollback-next"
      mv -Tf "$base/.rollback-next" "$base/current"
      systemctl restart airradarv2-engine || true
      systemctl restart nginx || true
    else
      systemctl stop airradarv2-engine nginx || true
      rm -f "$base/current"
    fi
  fi
  if [ -d "$work" ]; then rm -rf -- "$work"; fi
  exit "$exit_code"
}
trap 'rollback "$?"' ERR

tar -xzf "$bundle" -C "$work" --no-same-owner
test -s "$work/VERSION"
test -x "$work/airradarv2-engine"
test -s "$work/web/index.html"
sha="$(tr -d '\r\n' < "$work/VERSION")"
if [[ ! "$sha" =~ ^[0-9a-f]{40}$ ]]; then
  echo "ERROR: neplatný commit SHA v balíčku" >&2
  exit 1
fi
tag="$(printf '%s' "$sha" | cut -c1-12)"
target="$base/releases/$tag"
if [ ! -e "$target" ]; then
  chown -R root:root "$work"
  chmod 0755 "$work" "$work/airradarv2-engine"
  mv "$work" "$target"
else
  if [ "$(tr -d '\r\n' < "$target/VERSION")" != "$sha" ]; then
    echo "ERROR: konflikt release jména a SHA" >&2
    exit 1
  fi
  rm -rf -- "$work"
fi

ln -sfn "$target" "$base/.current-next"
mv -Tf "$base/.current-next" "$base/current"
activated=1
systemctl daemon-reload
systemctl enable --now airradarv2-engine
systemctl restart airradarv2-engine
nginx -t
systemctl enable --now nginx
systemctl reload nginx

# Health musí explicitně potvrdit stav bootstrap (bez falešných live dat).
ready=0
for attempt in $(seq 1 20); do
  if curl -fsS --max-time 2 http://127.0.0.1:4101/api/v1/health |
      grep -q '"status":"bootstrap"'; then
    ready=1
    break
  fi
  sleep 1
done
if [ "$ready" != 1 ]; then
  echo "ERROR: Rust API neodpovědělo bootstrap health" >&2
  rollback 1
fi
curl -fsS --max-time 4 http://127.0.0.1:4101/api/v1/health |
  grep -q '"live_ingest":false'
bind_ip="$(sed -nE 's/^[[:space:]]*listen ([0-9.]+):8080;.*$/\1/p' \
  /etc/nginx/sites-available/airradarv2-dev | head -n 1)"
test -n "$bind_ip"
curl -fsS --max-time 5 "http://$bind_ip:8080/" |
  grep -q '<div id="root"></div>'
systemctl is-active --quiet airradarv2-engine
systemctl is-active --quiet nginx
trap - ERR
echo "PASS: DEV AirRadar V2 SHA=$sha"
echo "LAN web: http://$bind_ip:8080/"
echo "Lokální API: http://127.0.0.1:4101/api/v1/health"
echo "G-02 ADS-B ani G-04 PostgreSQL nejsou v tomto buildu aktivní."
