#!/usr/bin/env bash
# Jednorázová aktivace bezpečného GitHub main -> samostatný AirRadar V2 DEV LXC.
# Pouze uvnitř lo-dev-airradarv2 jako root, nikoli na Proxmox hostu/V1.
set -Eeuo pipefail
if (( EUID != 0 )) || [[ "$#" -ne 0 ]]; then
  echo "ERROR: jen root uvnitř AirRadarV2 DEV LXC (bez parametrů)" >&2
  exit 2
fi
source /etc/os-release
if [[ "$ID" != debian || "$VERSION_ID" != 13 ||
      "$(uname -m)" != x86_64 || "$(hostname)" != lo-dev-airradarv2 ]]; then
  echo "ERROR: chybné prostředí; produktový/V1 host nesmí být změněn" >&2
  exit 1
fi
[[ -v G13_SOURCE_REF && "$G13_SOURCE_REF" =~ ^[0-9a-f]{40}$ ]] || {
  echo "ERROR: G13_SOURCE_REF musí obsahovat ověřené celé SHA instalátoru" >&2
  exit 1
}
test -f /etc/systemd/system/airradarv2-engine.service
test -f /etc/nginx/sites-available/airradarv2-dev
test -L /opt/airradar-v2/current
test -x /opt/airradar-v2/current/airradarv2-engine
test -f /opt/airradar-v2/current/VERSION
test -x /usr/bin/python3
test -x /usr/bin/flock
systemctl is-active --quiet nginx.service
systemctl is-active --quiet airradarv2-engine.service
[[ "$(systemctl show -p User --value airradarv2-engine.service)" == airradarv2 ]] || exit 1
grep -qF 'ExecStart=/opt/airradar-v2/current/airradarv2-engine' \
  /etc/systemd/system/airradarv2-engine.service || exit 1

tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT
base="https://raw.githubusercontent.com/Boym323/AirRadarV2/$G13_SOURCE_REF/deploy"
curl -fsSL --max-time 30 --proto '=https' --proto-redir '=https' \
  "$base/main-dev-auto-pull.py" -o "$tmp/agent.py"
curl -fsSL --max-time 30 --proto '=https' --proto-redir '=https' \
  "$base/install-dev-bundle.sh" -o "$tmp/installer.sh"
python3 -m py_compile "$tmp/agent.py"
bash -n "$tmp/installer.sh"
install -d -o root -g root -m 0755 /usr/local/libexec
install -o root -g root -m 0755 "$tmp/agent.py" \
  /usr/local/libexec/airradarv2-main-deploy-pull.py
install -o root -g root -m 0755 "$tmp/installer.sh" \
  /usr/local/libexec/airradarv2-main-deploy-install.sh
install -d -o root -g root -m 0700 /var/lib/airradar-v2/main-deploy
install -d -o root -g root -m 0755 /var/cache/airradarv2-main-deploy
if [[ ! -e /var/cache/airradarv2-main-deploy/status.json ]]; then
  printf '%s\n' '{"stage":"AirRadarV2 DEV main deployment","status":"WAITING_FOR_FIRST_POLL","deployedSha":null}' \
    >/var/cache/airradarv2-main-deploy/status.json
  chmod 0644 /var/cache/airradarv2-main-deploy/status.json
fi

# Jediná známá Nginx úprava: zcela read-only status bez interních údajů.
site=/etc/nginx/sites-available/airradarv2-dev
if ! grep -qF 'location = /dev-deploy-status.json {' "$site"; then
  grep -qF '    location /assets/ {' "$site"
  cp -p "$site" "$site.before-main-auto"
  python3 - "$site" <<'PY'
from pathlib import Path
import sys
p=Path(sys.argv[1])
s=p.read_text(encoding="utf-8")
anchor="    location /assets/ {"
assert s.count(anchor)==1
part="""    location = /dev-deploy-status.json {
        alias /var/cache/airradarv2-main-deploy/status.json;
        default_type application/json;
        add_header Cache-Control "no-store" always;
    }
"""
p.write_text(s.replace(anchor,part+anchor,1),encoding="utf-8")
PY
  if ! nginx -t; then
    cp -p "$site.before-main-auto" "$site"
    echo "ERROR: Nginx syntax selhala, změna vrácena" >&2
    exit 1
  fi
  systemctl reload nginx.service
fi

cat >/etc/systemd/system/airradarv2-main-deploy.service <<'SERVICE'
[Unit]
Description=AirRadar V2 DEV — ověřený automatický deploy posledního main SHA
After=network-online.target
Wants=network-online.target
ConditionPathExists=/usr/local/libexec/airradarv2-main-deploy-pull.py

[Service]
Type=oneshot
User=root
Group=root
WorkingDirectory=/var/lib/airradar-v2/main-deploy
ExecStart=/usr/bin/flock -n /run/lock/airradarv2-main-deploy.lock /usr/bin/python3 /usr/local/libexec/airradarv2-main-deploy-pull.py
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/opt/airradar-v2 /var/lib/airradar-v2/main-deploy /var/cache/airradarv2-main-deploy /run/lock /run/systemd /var/log/nginx
MemoryMax=768M
CPUQuota=60%
CPUWeight=10
TasksMax=128
TimeoutStartSec=300
UMask=0027
Environment=PYTHONDONTWRITEBYTECODE=1
SERVICE
cat >/etc/systemd/system/airradarv2-main-deploy.timer <<'TIMER'
[Unit]
Description=AirRadar V2 DEV — automatická aktualizace z ověřeného GitHub main
[Timer]
OnBootSec=3min
OnUnitInactiveSec=5min
RandomizedDelaySec=20s
Persistent=true
Unit=airradarv2-main-deploy.service
[Install]
WantedBy=timers.target
TIMER
systemctl daemon-reload
systemctl enable --now airradarv2-main-deploy.timer
echo "PASS: GitHub main auto-deploy na DEV LXC aktivován; V1/DB/přijímač nedotčeny."
systemctl list-timers --all 'airradarv2-main-deploy*'
echo "Status pouze pro čtení: /dev-deploy-status.json"
