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
incoming=/var/lib/airradarv2-runner/incoming
[[ "$bundle" == "$incoming"/*/* ]] || {
  echo "Artefakt je mimo izolovaný incoming adresář" >&2
  exit 1
}
staging_dir="${bundle%/*}"
staging_name="${staging_dir##*/}"
[[ "$staging_name" =~ ^[0-9a-f]{40}\.[0-9]+$ ]] || {
  echo "Artefakt nemá izolovaný SHA/run staging adresář" >&2
  exit 1
}
[[ "$(stat -c '%U:%G %a' -- "$staging_dir")" == "airradarv2-runner:airradarv2-runner 700" ]] || {
  echo "Staging adresář nemá očekávaného vlastníka nebo režim" >&2
  exit 1
}
test -f "$bundle.sha256"

exec 9>/run/lock/airradarv2-main-deploy.lock
if ! flock -n 9; then
  echo "Deployment lock je obsazený jiným nasazením nebo G-02; opakovatelný konflikt (75)." >&2
  exit 75
fi
for service in airradarv2-g02-auto.service airradarv2-g02-live-auto.service; do
  if systemctl is-active --quiet "$service"; then
    echo "$service právě běží; deployment bude opakován (75), nikoli vydáván za chybu G-02" >&2
    exit 75
  fi
done

export AIRRADAR_V2_AUTO_DEPLOY=1
export AIRRADAR_EXPECTED_SHA="$expected"
/usr/bin/bash /usr/local/libexec/airradarv2-main-deploy-install.sh "$bundle"

# Runner deployment does not use the disabled legacy polling timer. Publish
# the SHA that the installer actually activated, atomically and without any
# credentials or internal paths.
/usr/bin/python3 - "$expected" <<'PY'
import datetime as dt
import json
import os
from pathlib import Path
import re
import sys

expected = sys.argv[1]
sha_path = Path('/opt/airradar-v2/current/VERSION')
public = Path('/var/cache/airradarv2-main-deploy/status.json')
sha = sha_path.read_text(encoding='ascii').strip()
if not re.fullmatch(r'[0-9a-f]{40}', sha) or sha != expected:
    raise SystemExit('nasazeny VERSION neodpovida ocekavanemu SHA')
body = {
    'stage': 'AirRadarV2 DEV main deployment',
    'status': 'DEPLOYED',
    'deployedSha': sha,
    'candidateSha': sha,
    'checkedAtUtc': dt.datetime.now(dt.timezone.utc).isoformat(),
}
public.parent.mkdir(parents=True, exist_ok=True)
temporary = public.with_suffix('.json.tmp')
temporary.write_text(json.dumps(body, sort_keys=True, ensure_ascii=False) + '\n', encoding='utf-8')
os.chmod(temporary, 0o644)
temporary.replace(public)
PY
