#!/usr/bin/env bash
# G03_STAGE_GATE_V1 — strict stage-aware SSE health/rollback. G00 stays unchanged.
set -Eeuo pipefail
if [ "$(id -u)" -ne 0 ] || [ "$#" -ne 1 ]; then
  echo "Použití: bash install-dev-bundle.sh /root/airradarv2-dev-linux-amd64-....tar.gz" >&2
  exit 2
fi
source /etc/os-release
[[ "$ID" == debian && "$VERSION_ID" == 13 && "$(uname -m)" == x86_64 ]] || {
  echo "ERROR: instalace pouze do samostatného Debian 13 x86_64 DEV LXC" >&2
  exit 1
}
AUTO=0
if [[ -v AIRRADAR_V2_AUTO_DEPLOY ]]; then AUTO="$AIRRADAR_V2_AUTO_DEPLOY"; fi
[[ "$AUTO" == 0 || "$AUTO" == 1 ]] || exit 1
if [[ "$AUTO" == 1 ]]; then
  [[ -v AIRRADAR_EXPECTED_SHA && "$AIRRADAR_EXPECTED_SHA" =~ ^[0-9a-f]{40}$ ]] || {
    echo "ERROR: automatické nasazení vyžaduje očekávaný immutable SHA" >&2
    exit 1
  }
  systemctl is-active --quiet airradarv2-engine.service
  systemctl is-active --quiet nginx.service
  [[ "$(systemctl show -p User --value airradarv2-engine.service)" == airradarv2 ]] || exit 1
  [[ "$(systemctl show -p FragmentPath --value airradarv2-engine.service)" == "/etc/systemd/system/airradarv2-engine.service" ]] || exit 1
  grep -qF 'ExecStart=/opt/airradar-v2/current/airradarv2-engine' /etc/systemd/system/airradarv2-engine.service
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
# G-13: canonical validator rejects traversal, links and special files.
expected_for_validation="${AIRRADAR_EXPECTED_SHA:-$(tar -xOzf "$bundle" ./VERSION 2>/dev/null | tr -d '\r\n')}"
python3 - "$bundle" "$expected_for_validation" <<'PY'
import pathlib, sys
import tarfile  # explicit dependency retained for the archive safety gate
sys.path.insert(0, "/usr/local/libexec")
from archive_validation import validate_archive
p = pathlib.Path(sys.argv[1])
if p.stat().st_size > 150 * 1024 * 1024:
    raise SystemExit("ERROR: compressed bundle exceeds 150 MiB")
validate_archive(p.read_bytes(), sys.argv[2])
PY
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
  [[ "$old" == "$base/releases/"* && -d "$old" ]] || { echo "ERROR: current mimo DEV releases" >&2; exit 1; }
fi
activated=0
health_tmp="$(mktemp)"
web_tmp="$(mktemp)"

rollback() {
  exit_code="$1"
  trap - EXIT
  if [ "$exit_code" -ne 0 ]; then
    echo "ERROR: instalace nebo health check selhal; vracím předchozí verzi." >&2
  if [ "$activated" -eq 1 ]; then
    if [ -n "$old" ] && [ -d "$old" ]; then
      ln -sfn "$old" "$base/.rollback-next"
      mv -Tf "$base/.rollback-next" "$base/current"
      systemctl restart airradarv2-engine || true
      if nginx -t; then systemctl reload nginx || true; fi
    else
      systemctl stop airradarv2-engine nginx || true
      rm -f "$base/current"
    fi
  fi
  fi
  if [ -d "$work" ]; then rm -rf -- "$work"; fi
  rm -f -- "$health_tmp" "$web_tmp"
  exit "$exit_code"
}
trap 'rollback "$?"' EXIT
trap 'exit 124' TERM INT # ensure EXIT rollback on systemd stop/timeout

tar -xzf "$bundle" -C "$work" --no-same-owner
test -s "$work/VERSION"
test -x "$work/airradarv2-engine"
test -s "$work/web/index.html"
sha="$(tr -d '\r\n' < "$work/VERSION")"
if [[ ! "$sha" =~ ^[0-9a-f]{40}$ ]]; then
  echo "ERROR: neplatný commit SHA v balíčku" >&2
  exit 1
fi
if [[ "$AUTO" == 1 && "$sha" != "$AIRRADAR_EXPECTED_SHA" ]]; then
  echo "ERROR: VERSION se neshoduje se schváleným SHA" >&2
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
  # Do not delete the staging directory until healthcheck has used it.
fi

# G03_STAGE_GATE_V1: verify the approved systemd mode & installed verifier
# before switching symlink. Without root-owned gate, refuse G03 deployment.
requires_g03=0
if [[ -f "$target/web/g03-runtime-required.txt" ]]; then
  test "$(tr -d '\r\n' < "$target/web/g03-runtime-required.txt")" == "G-03"
  test -r /usr/local/libexec/airradarv2-verify-g03-runtime.py
  grep -qF -- '--g03-live-api' /etc/systemd/system/airradarv2-engine.service || {
    echo "ERROR: G-03 release vyžaduje aktivovaný systemd --g03-live-api; neprovádím cutover" >&2
    exit 1
  }
  requires_g03=1
fi

ln -sfn "$target" "$base/.current-next"
mv -Tf "$base/.current-next" "$base/current"
activated=1
if [[ "$AUTO" == 1 ]]; then
  systemctl restart airradarv2-engine
else
  systemctl daemon-reload
  systemctl enable --now airradarv2-engine
  systemctl restart airradarv2-engine
fi
nginx -t
if [[ "$AUTO" != 1 ]]; then
  systemctl enable --now nginx
fi
systemctl reload nginx

# Health check accepts G-00 bootstrap or explicit G-03+ state (never stale fake live).
ready=0
for attempt in $(seq 1 25); do
  if curl -fsS --max-time 3 http://127.0.0.1:4101/api/v1/health -o "$health_tmp" 2>/dev/null &&
     python3 - "$health_tmp" "$requires_g03" <<'PY'
import json,sys
try:
    with open(sys.argv[1],encoding="utf-8") as f:
        d=json.load(f)
    if sys.argv[2] == "1":
        assert d.get("stage") == "G-03"
        assert d.get("status") in ("live", "waiting_for_observations")
        assert d.get("live_ingest") is (d["status"] == "live")
    else:
        assert d.get("stage") == "G-00"
        assert d.get("status") == "bootstrap"
        assert d.get("live_ingest") is False
    assert d.get("database_connected") is False
except (OSError,ValueError,KeyError,AssertionError,TypeError):
    sys.exit(1)
PY
  then
    ready=1
    break
  fi
  sleep 1
done
if [[ "$ready" != 1 ]]; then
  echo "ERROR: validní DEV health nedostupný" >&2
  exit 1
fi
bind_ip="$(sed -nE 's/^[[:space:]]*listen ([0-9.]+):8080;.*$/\1/p' \
  /etc/nginx/sites-available/airradarv2-dev | head -n 1)"
test -n "$bind_ip"
curl -fsS --max-time 5 "http://$bind_ip:8080/" |
  grep -q '<div id="root"></div>'
if [[ "$requires_g03" == 1 ]]; then
  # Still inside the EXIT rollback trap: a missing/blocked proxy SSE stream
  # restores the previous release instead of marking a false deployment PASS.
  python3 /usr/local/libexec/airradarv2-verify-g03-runtime.py \
    --direct http://127.0.0.1:4101 \
    --proxy "http://$bind_ip:8080"
fi
systemctl is-active --quiet airradarv2-engine
systemctl is-active --quiet nginx
echo "PASS: DEV AirRadar V2 SHA=$sha"
echo "LAN web: http://$bind_ip:8080/"
echo "Lokální API: http://127.0.0.1:4101/api/v1/health"
echo "G-02 ADS-B ani G-04 PostgreSQL nejsou v tomto buildu aktivní."
