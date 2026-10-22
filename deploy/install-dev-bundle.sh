#!/usr/bin/env bash
# G03_STAGE_GATE_V2 — atomic systemd mode + immutable release switch + rollback.
set -Eeuo pipefail
if [ "$(id -u)" -ne 0 ] || [ "$#" -ne 1 ]; then
  echo "Použití: bash install-dev-bundle.sh /root/airradarv2-dev-linux-amd64-....tar.gz" >&2
  exit 2
fi
source /etc/os-release
[[ "$ID" == debian && "$VERSION_ID" == 13 && "$(uname -m)" == x86_64 && "$(hostname)" == lo-dev-airradarv2 ]] || {
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
mode_changed=0
mode_dir=/etc/systemd/system/airradarv2-engine.service.d
mode_file="$mode_dir/g03-auto.conf"
mode_backup="$(mktemp)"
mode_was_present=0
if [[ -f "$mode_file" ]]; then
  cp -- "$mode_file" "$mode_backup"
  mode_was_present=1
fi
health_tmp="$(mktemp)"
web_tmp="$(mktemp)"

rollback() {
  exit_code="$1"
  trap - EXIT
  if [ "$exit_code" -ne 0 ]; then
    echo "ERROR: instalace nebo health check selhal; vracím předchozí verzi." >&2
    # Obnovit v jedné rollback transakci verzi binárky i původní
    # systemd režim. G-00 binárka nesmí být restartována s --g03-live-api.
    if [ "$activated" -eq 1 ]; then
      if [ -n "$old" ] && [ -d "$old" ]; then
        ln -sfn "$old" "$base/.rollback-next"
        mv -Tf "$base/.rollback-next" "$base/current"
      else
        systemctl stop airradarv2-engine nginx || true
        rm -f "$base/current"
      fi
    fi
    if [ "$mode_changed" -eq 1 ]; then
      if [ "$mode_was_present" -eq 1 ]; then
        install -o root -g root -m 0644 "$mode_backup" "$mode_file"
      else
        rm -f -- "$mode_file"
      fi
      systemctl daemon-reload || true
    fi
    if [ "$activated" -eq 1 ] && [ -n "$old" ] && [ -d "$old" ]; then
      systemctl restart airradarv2-engine || true
      if nginx -t; then systemctl reload nginx || true; fi
    fi
  fi
  if [ -d "$work" ]; then rm -rf -- "$work"; fi
  rm -f -- "$health_tmp" "$web_tmp" "$mode_backup"
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

# G03_STAGE_GATE_V2: auto-activate the runtime after validated bundle;
# the root-owned bridge owns systemd and can roll back *both* release and mode.
requires_g03=0
if [[ -f "$target/web/g03-runtime-required.txt" ]]; then
  test "$(tr -d '\r\n' < "$target/web/g03-runtime-required.txt")" == "G-03"
  test -r /usr/local/libexec/airradarv2-verify-g03-runtime.py
  requires_g03=1
fi
# Nezbytně nejprve ověřit skutečný zdroj read-only BEAST již dříve
# schválený a testovaný v G-02; nikdy nehledat receiver ve V1 configu.
if [[ "$requires_g03" -eq 1 ]]; then
  source_unit=/etc/systemd/system/airradarv2-g02-live-auto.service
  test -r "$source_unit" || {
    echo "G-03: chybí již nakonfigurovaná G-02 read-only BEAST služba." >&2
    exit 1
  }
  receiver="$(sed -nE 's/^Environment=AIRRADAR_V2_G02_RECEIVER=([0-9.]+)$/\1/p' "$source_unit" | head -n1)"
  port="$(sed -nE 's/^Environment=AIRRADAR_V2_G02_PORT=([0-9]+)$/\1/p' "$source_unit" | head -n1)"
  test "$port" = 30005
  python3 - "$receiver" <<'PY'
import ipaddress, sys
try:
    ip = ipaddress.IPv4Address(sys.argv[1])
    assert ip.is_private and not ip.is_loopback and not ip.is_link_local
except (ValueError, IndexError, AssertionError):
    raise SystemExit("G-03: přijímač nemá validní soukromou IPv4")
PY
fi

install -d -o root -g root -m 0755 "$mode_dir"
if [[ "$requires_g03" -eq 1 ]]; then
  mode_next="$(mktemp "$mode_dir/.g03-auto.XXXXXXXX")"
  cat > "$mode_next" <<UNIT
# G-03 auto-managed by the root-owned DEV installer; do not hand-edit.
[Service]
Environment=AIRRADAR_V2_BEAST_ADDR=$receiver:30005
ExecStart=
ExecStart=/opt/airradar-v2/current/airradarv2-engine --g03-live-api
UNIT
  chmod 0644 "$mode_next"
  mode_changed=1
  mv -f -- "$mode_next" "$mode_file"
else
  # Older G-00 release, e.g. for a deliberate rollback, must not inherit
  # the G-03 ExecStart override.
  mode_changed=1
  rm -f -- "$mode_file"
fi
systemctl daemon-reload

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
if [[ "$requires_g03" == 1 ]]; then
  echo "G-03 HTTP/SSE je aktivní nad read-only BEAST, DB G-04 zůstává vypnutá."
else
  echo "G-00 bootstrap bez G-03; DB G-04 zůstává vypnutá."
fi
