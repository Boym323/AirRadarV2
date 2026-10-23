#!/usr/bin/env bash
# AirRadar V2 — pouze izolovaný DEV LXC, G-00/G-01. Bez produkce, DB a ADS-B.
set -Eeuo pipefail
if [ "$(id -u)" -ne 0 ] || [ "$#" -ne 2 ]; then
  echo "Použití: bash bootstrap-dev-lxc.sh <privátní IPv4 LXC> <povolená LAN CIDR>" >&2
  exit 2
fi
BIND_IP="$1"
ALLOW_NET="$2"
source /etc/os-release
if [ "$ID" != "debian" ] || [ "$VERSION_ID" != "13" ] || [ "$(uname -m)" != x86_64 ]; then
  echo "ERROR: vyžaduje Debian 13 x86_64, nic neměním" >&2
  exit 1
fi
if [[ ! "$BIND_IP" =~ ^[0-9]{1,3}(\.[0-9]{1,3}){3}$ ]] ||
   [[ ! "$ALLOW_NET" =~ ^[0-9]{1,3}(\.[0-9]{1,3}){3}/[0-9]{1,2}$ ]]; then
  echo "ERROR: neplatná adresa IPv4/CIDR" >&2
  exit 1
fi
if ! ip -o -4 addr show | grep -qF "$BIND_IP/"; then
  echo "ERROR: uvedená IPv4 není přiřazena tomuto LXC" >&2
  exit 1
fi
if [[ "$BIND_IP" != 10.* && "$BIND_IP" != 192.168.* ]] &&
   [[ ! "$BIND_IP" =~ ^172\.(1[6-9]|2[0-9]|3[01])\. ]]; then
  echo "ERROR: DEV musí používat privátní RFC1918 adresu" >&2
  exit 1
fi
net_mask="$(printf '%s' "$ALLOW_NET" | cut -d/ -f2)"
if [ "$net_mask" -lt 16 ] || [ "$net_mask" -gt 32 ]; then
  echo "ERROR: DEV síť musí mít masku /16 až /32, nikoliv otevřený internet" >&2
  exit 1
fi
if systemctl is-active --quiet airradarv2-engine; then
  echo "ERROR: aplikace již běží, neopakuj bootstrap; použij instalační skript" >&2
  exit 1
fi
for section in $(echo "$BIND_IP" | tr '.' ' '); do
  if [ "$section" -gt 255 ]; then echo "ERROR: neplatná IPv4" >&2; exit 1; fi
done

echo "Instaluji jen runtime balíčky nginx, curl, CA — bez Node/Rust/DB."
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y --no-install-recommends nginx ca-certificates curl
systemctl stop nginx 2>/dev/null || true

if [ -L /etc/nginx/sites-enabled/default ]; then
  if [ "$(readlink /etc/nginx/sites-enabled/default)" != "/etc/nginx/sites-available/default" ]; then
    echo "ERROR: nginx default odkaz není standardní; nic nepřepisuji" >&2
    exit 1
  fi
  rm /etc/nginx/sites-enabled/default
fi
for site in /etc/nginx/sites-enabled/*; do
  if [ -e "$site" ] || [ -L "$site" ]; then
    if [ "$site" != /etc/nginx/sites-enabled/airradarv2-dev ]; then
      echo "ERROR: nalezena další nginx site: $site" >&2
      exit 1
    fi
  fi
done

if ! id airradarv2 >/dev/null 2>&1; then
  useradd --system --user-group --home-dir /var/lib/airradar-v2 \
    --create-home --shell /usr/sbin/nologin airradarv2
fi
install -d -o root -g root -m 0755 /opt/airradar-v2 /opt/airradar-v2/releases
install -d -o airradarv2 -g airradarv2 -m 0750 /var/lib/airradar-v2

cat > /etc/systemd/system/airradarv2-engine.service <<'SERVICE'
[Unit]
Description=AirRadar V2 DEV — Rust G-00/G-01 API
After=network-online.target
Wants=network-online.target
ConditionPathExists=/opt/airradar-v2/current/airradarv2-engine

[Service]
Type=simple
User=airradarv2
Group=airradarv2
WorkingDirectory=/opt/airradar-v2/current
Environment=AIRRADAR_V2_BIND=127.0.0.1:4101
ExecStart=/opt/airradar-v2/current/airradarv2-engine
Restart=on-failure
RestartSec=3s
NoNewPrivileges=yes
UMask=0027
LimitNOFILE=4096
MemoryMax=2G
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
SERVICE

install -d -o root -g root -m 0755 /etc/systemd/system/nginx.service.d
install -o root -g root -m 0644 deploy/systemd/nginx-airradarv2.conf \
  /etc/systemd/system/nginx.service.d/airradarv2.conf

cat > /etc/nginx/sites-available/airradarv2-dev <<'NGINX'
# DEV: pouze privátní IPv4; žádné naslouchání na globální IPv6.
server {
    listen __BIND_IP__:8080;
    server_name _;
    root /opt/airradar-v2/current/web;
    index index.html;
    autoindex off;
    client_max_body_size 1m;

    allow __ALLOW_NET__;
    deny all;
    add_header X-Content-Type-Options nosniff always;
    add_header Referrer-Policy no-referrer always;

    location = /api/v1/stream {
        proxy_pass http://127.0.0.1:4101;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache off;
        proxy_set_header X-Accel-Buffering no;
        add_header X-Accel-Buffering no always;
        add_header Cache-Control "no-cache, no-transform" always;
        proxy_read_timeout 3600s;
    }
    location /api/ {
        proxy_pass http://127.0.0.1:4101;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_read_timeout 30s;
    }
    location /assets/ {
        try_files $uri =404;
        expires 7d;
    }
    location / {
        try_files $uri $uri/ /index.html;
        add_header Cache-Control "no-store";
    }
}
NGINX

sed -i "s/__BIND_IP__/$BIND_IP/g; s@__ALLOW_NET__@$ALLOW_NET@g" \
  /etc/nginx/sites-available/airradarv2-dev
ln -sfn /etc/nginx/sites-available/airradarv2-dev \
  /etc/nginx/sites-enabled/airradarv2-dev
nginx -t
systemctl daemon-reload
# Služby aktivuje teprve instalační skript po úspěšném ověření artefaktu.
systemctl disable nginx 2>/dev/null || true
echo "PASS: Runtime LXC připraven; zatím není nasazena aplikace."
echo "Přeneste ověřený CI archiv + .sha256 a spusťte install-dev-bundle.sh."
