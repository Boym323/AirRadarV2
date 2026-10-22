#!/usr/bin/env bash
# Jednorázové povolení root-owned G-03 DEV installeru pro automatický main→LXC.
# Neregistruje runner znovu, nemění běžící service, V1, DB ani receiver.
set -Eeuo pipefail
(( EUID == 0 )) || { echo "G-03: spusť uvnitř DEV LXC jako root." >&2; exit 2; }
source /etc/os-release
[[ "$ID" == debian && "$VERSION_ID" == 13 && "$(uname -m)" == x86_64 && "$(hostname)" == lo-dev-airradarv2 ]] || {
  echo "G-03: pouze oddělený Debian 13 AirRadarV2 DEV." >&2; exit 1;
}
repo="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$repo"
test -d .git
test "$(git rev-parse --abbrev-ref HEAD)" = main
test "$(git rev-parse HEAD)" = "$(git rev-parse refs/remotes/origin/main)" || {
  echo "G-03: nejprve git fetch origin main && git pull --ff-only origin main." >&2
  exit 1
}
test -z "$(git status --porcelain)" || { echo "G-03: checkout má lokální změny." >&2; exit 1; }
for p in deploy/install-dev-bundle.sh deploy/runner-deploy.sh deploy/archive_validation.py deploy/verify-g03-runtime.py; do
  test -f "$p"
  test "$(stat -c '%u' "$p")" = 0
  permissions="$(stat -c '%a' "$p")"
  test $(( 8#$permissions & 0022 )) -eq 0
done
grep -qF G03_STAGE_GATE_V2 deploy/install-dev-bundle.sh
bash -n deploy/install-dev-bundle.sh deploy/runner-deploy.sh
python3 - <<'PY'
from pathlib import Path
for p in ("deploy/archive_validation.py", "deploy/verify-g03-runtime.py"):
    compile(Path(p).read_bytes(), p, "exec")
PY
test -x /opt/airradar-v2/current/airradarv2-engine
test -f /etc/systemd/system/airradarv2-engine.service
test -L /etc/nginx/sites-enabled/airradarv2-dev
systemctl is-active --quiet airradarv2-engine.service
systemctl is-active --quiet nginx.service
systemctl is-active --quiet actions.runner.Boym323-AirRadarV2.airradarv2-dev.service
test "$(systemctl show --value -p User airradarv2-engine.service)" = airradarv2
grep -qF 'ExecStart=/opt/airradar-v2/current/airradarv2-engine' /etc/systemd/system/airradarv2-engine.service
unit=/etc/systemd/system/airradarv2-g02-live-auto.service
test -r "$unit"
receiver="$(sed -nE 's/^Environment=AIRRADAR_V2_G02_RECEIVER=([0-9.]+)$/\1/p' "$unit" | head -n1)"
port="$(sed -nE 's/^Environment=AIRRADAR_V2_G02_PORT=([0-9]+)$/\1/p' "$unit" | head -n1)"
test "$port" = 30005
python3 - "$receiver" <<'PY'
import ipaddress, sys
try:
    ip = ipaddress.IPv4Address(sys.argv[1])
    assert ip.is_private and not ip.is_loopback and not ip.is_link_local
except (ValueError, IndexError, AssertionError):
    raise SystemExit("G-03: neplatná soukromá IPv4 dříve schváleného read-only BEAST přijímače")
PY
nginx -t
visudo -cf /etc/sudoers.d/airradarv2-runner
test -d /var/lib/airradarv2-runner/incoming
install -o root -g root -m 0644 deploy/archive_validation.py /usr/local/libexec/archive_validation.py
install -o root -g root -m 0755 deploy/verify-g03-runtime.py /usr/local/libexec/airradarv2-verify-g03-runtime.py
install -o root -g root -m 0755 deploy/install-dev-bundle.sh /usr/local/libexec/airradarv2-main-deploy-install.sh
install -o root -g root -m 0755 deploy/runner-deploy.sh /usr/local/libexec/airradarv2-runner-deploy.sh
grep -qF G03_STAGE_GATE_V2 /usr/local/libexec/airradarv2-main-deploy-install.sh
echo "PASS: root-owned G-03 installer připraven, další main commit se automaticky nasadí."
echo "Aktuální engine/systemd/nginx zůstávají beze změny až do GitHub release."
