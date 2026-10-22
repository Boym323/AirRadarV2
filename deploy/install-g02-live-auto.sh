#!/usr/bin/env bash
# G-02.11: one-time installer of outbound-only GitHub live acceptance poller.
# Usage: G02_SOURCE_REF=verified_full_sha bash install-g02-live-auto.sh 192.168.1.50
set -Eeuo pipefail
if (( EUID != 0 )) || [[ "$#" -ne 1 ]]; then
  echo "Run as root on V2 DEV LXC with full G02_SOURCE_REF SHA and receiver IP" >&2
  exit 2
fi
source /etc/os-release
if [[ "$ID" != "debian" || "$VERSION_ID" != 13 || "$(uname -m)" != "x86_64" ]]; then
  echo "G-02: only Debian 13 x86_64 separate DEV LXC" >&2
  exit 1
fi
[[ "$G02_SOURCE_REF" =~ ^[0-9a-f]{40}$ ]] || {
  echo "G-02: G02_SOURCE_REF must be immutable full SHA" >&2
  exit 1
}
test -f /etc/systemd/system/airradarv2-g02-auto.service
test -x /opt/airradar-v2/current/airradarv2-engine
test -x /usr/local/libexec/airradarv2-g02-pull.py
test -r /var/cache/airradarv2-g02-public/status.json
id airradarv2 >/dev/null
python3 - "$1" <<'PY'
import ipaddress, sys
ip = ipaddress.IPv4Address(sys.argv[1])
if not ip.is_private or ip.is_loopback or ip.is_link_local:
    raise SystemExit("G-02: only private IPv4 receiver")
PY
receiver="$1"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
base="https://raw.githubusercontent.com/Boym323/AirRadarV2/$G02_SOURCE_REF/deploy"
curl -fsSL --max-time 35 "$base/g02-live-soak.py" -o "$stage/live.py"
curl -fsSL --max-time 35 "$base/g02-live-auto.py" -o "$stage/auto.py"
python3 -m py_compile "$stage/live.py" "$stage/auto.py"
install -o root -g root -m 0755 "$stage/live.py" /usr/local/libexec/airradarv2-g02-live-soak.py
install -o root -g root -m 0755 "$stage/auto.py" /usr/local/libexec/airradarv2-g02-live-auto.py

# Fixed local operations only; GitHub releases carry intent, never arbitrary shell.
cat > /etc/systemd/system/airradarv2-g02-live-auto.service <<SERVICE
[Unit]
Description=AirRadar V2 G-02.11 GitHub-requested read-only BEAST soak
After=network-online.target
Wants=network-online.target
ConditionPathExists=/usr/local/libexec/airradarv2-g02-live-auto.py

[Service]
Type=oneshot
User=airradarv2
Group=airradarv2
WorkingDirectory=/var/lib/airradar-v2/g02-auto
Environment=AIRRADAR_V2_G02_RECEIVER=$receiver
Environment=AIRRADAR_V2_G02_PORT=30005
ExecStart=/usr/bin/python3 /usr/local/libexec/airradarv2-g02-live-auto.py
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/airradar-v2/g02-auto /var/cache/airradarv2-g02-public
MemoryMax=384M
CPUQuota=50%
CPUWeight=10
TasksMax=64
RuntimeMaxSec=3900
UMask=0027
SERVICE
cat > /etc/systemd/system/airradarv2-g02-live-auto.timer <<'TIMER'
[Unit]
Description=AirRadar V2 G-02.11 GitHub request poll every 3 minutes
[Timer]
OnBootSec=2min
OnUnitInactiveSec=3min
RandomizedDelaySec=10s
Persistent=true
Unit=airradarv2-g02-live-auto.service
[Install]
WantedBy=timers.target
TIMER
systemctl daemon-reload
systemctl enable --now airradarv2-g02-live-auto.timer
echo "PASS: G-02 live GitHub agent installed (BEAST readonly $receiver:30005); V1 and active engine unchanged"
systemctl list-timers --all 'airradarv2-g02-live-auto*'
