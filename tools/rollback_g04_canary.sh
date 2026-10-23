#!/usr/bin/env bash
set -Eeuo pipefail

# Bezpečný návrat: zastaví pouze opt-in unit. Historii nemaže a G-03 nereštartuje.
systemctl stop airradarv2-g04-canary.service
test "$(systemctl show -p ActiveState --value airradarv2-g04-canary.service)" = inactive
systemctl is-active --quiet airradarv2-engine.service
grep -qF 'ExecStart=/opt/airradar-v2/current/airradarv2-engine --g03-live-api' \
  /etc/systemd/system/airradarv2-engine.service.d/*.conf
echo 'PASS: G-04 canary zastaven; G-03 unit zůstává aktivní; data nebyla mazána.'
