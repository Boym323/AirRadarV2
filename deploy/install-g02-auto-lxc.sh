#!/usr/bin/env bash
# Jednorazovy installer jen pro izolovany Debian 13 AirRadar V2 DEV LXC.
set -Eeuo pipefail
if [[ "$(id -u)" != 0 || "$#" -ne 1 ]]; then
  echo "Pouziti: sudo bash install-g02-auto-lxc.sh /cesta/g02-auto-pull.py" >&2
  exit 2
fi
source /etc/os-release
if [[ "$ID" != debian || "$VERSION_ID" != "13" || "$(uname -m)" != x86_64 ]]; then
  echo "ERROR: pouze Debian 13 x86_64 DEV LXC" >&2
  exit 1
fi
unit=/etc/systemd/system/airradarv2-engine.service
site=/etc/nginx/sites-available/airradarv2-dev
if [[ ! -f "$unit" || ! -f "$site" || ! -L /opt/airradar-v2/current ||
      ! -x /opt/airradar-v2/current/airradarv2-engine ]] ||
      ! id airradarv2 >/dev/null 2>&1; then
  echo "ERROR: chybí overeny AirRadar V2 DEV bootstrap" >&2
  exit 1
fi
if ! grep -qF 'Environment=AIRRADAR_V2_BIND=127.0.0.1:4101' "$unit"; then
  echo "ERROR: konfigurace engine neodpovida DEV bootstrapu" >&2
  exit 1
fi
command -v python3 >/dev/null || {
  echo "ERROR: pouze v DEV LXC nejdrive nainstaluj python3-minimal" >&2
  exit 1
}
test -r "$1"
python3 -m py_compile "$1"
install -D -o root -g root -m 0755 "$1" /usr/local/libexec/airradarv2-g02-pull.py
install -d -o airradarv2 -g airradarv2 -m 0750 /var/lib/airradar-v2/g02-auto
install -d -o airradarv2 -g airradarv2 -m 0755 /var/cache/airradarv2-g02-public
if [[ ! -f /var/cache/airradarv2-g02-public/status.json ]]; then
  printf '%s\n' '{"stage":"G-02.11","status":"NEOVERENO","message":"Ceka na prvni immutable DEV kandidat."}' \
    >/var/cache/airradarv2-g02-public/status.json
  chown airradarv2:airradarv2 /var/cache/airradarv2-g02-public/status.json
  chmod 0644 /var/cache/airradarv2-g02-public/status.json
fi

cat >/etc/systemd/system/airradarv2-g02-auto.service <<'SERVICE'
[Unit]
Description=AirRadar V2 G-02 read-only offline DEV acceptance
After=network-online.target
Wants=network-online.target
ConditionPathExists=/opt/airradar-v2/current/VERSION

[Service]
Type=oneshot
User=airradarv2
Group=airradarv2
WorkingDirectory=/var/lib/airradar-v2/g02-auto
ExecStart=/usr/bin/python3 /usr/local/libexec/airradarv2-g02-pull.py
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/airradar-v2/g02-auto /var/cache/airradarv2-g02-public
MemoryMax=768M
CPUWeight=10
TasksMax=64
RuntimeMaxSec=600
UMask=0027
SERVICE
cat >/etc/systemd/system/airradarv2-g02-auto.timer <<'TIMER'
[Unit]
Description=AirRadar V2 G-02 offline acceptance kazdych 15 minut

[Timer]
OnBootSec=3min
OnUnitActiveSec=15min
RandomizedDelaySec=30s
Persistent=true
Unit=airradarv2-g02-auto.service

[Install]
WantedBy=timers.target
TIMER

# Samostatna read-only cesta ve stavajicim LAN-only Nginx serveru.
if ! grep -qF 'location = /g02-status.json {' "$site"; then
  grep -qF '    location /assets/ {' "$site" || {
    echo "ERROR: neocekavana Nginx DEV konfigurace" >&2
    exit 1
  }
  cp -p "$site" "$site.before-g02-auto"
  python3 - "$site" <<'PY'
from pathlib import Path
import sys
p = Path(sys.argv[1])
text = p.read_text()
fragment = """    location = /g02-status.json {
        alias /var/cache/airradarv2-g02-public/status.json;
        default_type application/json;
        add_header Cache-Control "no-store" always;
    }
"""
p.write_text(text.replace("    location /assets/ {", fragment + "    location /assets/ {", 1))
PY
  if ! nginx -t; then
    cp -p "$site.before-g02-auto" "$site"
    nginx -t
    echo "ERROR: Nginx konfigurace vracena" >&2
    exit 1
  fi
  systemctl reload nginx
fi
systemctl daemon-reload
systemctl enable --now airradarv2-g02-auto.timer
echo "PASS: automaticke offline G-02 benchmarky zapnuty, V1 a bezici DEV engine nezmeneny."
echo "Overeni jednorazove: systemctl start airradarv2-g02-auto.service"
echo "Vysledky: /g02-status.json na stavajicim privatnim DEV webu"
