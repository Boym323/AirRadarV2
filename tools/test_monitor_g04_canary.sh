#!/usr/bin/env bash
set -Eeuo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bash -n "$root/tools/monitor_g04_canary.sh"
grep -q 'MemoryMax=2G' "$root/deploy/systemd/airradarv2-g04-canary.service"
grep -q 'CPUQuota=50%' "$root/deploy/systemd/airradarv2-g04-canary.service"
grep -q 'fail-closed' "$root/tools/monitor_g04_canary.sh"
echo 'PASS: G-04 monitor syntaxe, limity a fail-closed kontrakt'
