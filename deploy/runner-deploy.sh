#!/usr/bin/env bash
# Root-owned, narrowly scoped bridge called by the repo-only Actions runner.
set -Eeuo pipefail

if (( EUID != 0 )) || [[ "$#" -ne 2 ]]; then
  echo "Použití: runner-deploy.sh /cesta/bundle.tar.gz 40hexSHA" >&2
  exit 2
fi
bundle="$(realpath -e -- "$1")"
expected="$2"
[[ "$expected" =~ ^[0-9a-f]{40}$ ]] || exit 2
[[ "$bundle" == /var/lib/airradarv2-runner/incoming/* ]] || {
  echo "Artefakt je mimo izolovaný incoming adresář" >&2
  exit 1
}

exec 9>/run/lock/airradarv2-main-deploy.lock
if ! flock -n 9; then
  exit 75
fi
for service in airradarv2-g02-auto.service airradarv2-g02-live-auto.service; do
  if systemctl is-active --quiet "$service"; then
    echo "$service právě běží; deployment bude opakován" >&2
    exit 75
  fi
done

export AIRRADAR_V2_AUTO_DEPLOY=1
export AIRRADAR_EXPECTED_SHA="$expected"
exec /usr/bin/bash /usr/local/libexec/airradarv2-main-deploy-install.sh "$bundle"
