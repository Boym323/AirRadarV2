#!/usr/bin/env bash
# Nainstaluje runner AirRadar V2 a root-owned deployment bridge.
set -Eeuo pipefail
(( EUID == 0 )) || { echo "Spusť jako root uvnitř DEV LXC." >&2; exit 2; }
: "${RUNNER_REGISTRATION_TOKEN:?Nastav krátkodobý repo registration token.}"
source /etc/os-release
[[ "$ID" == debian && "$VERSION_ID" == 13 && "$(uname -m)" == x86_64 && "$(hostname)" == lo-dev-airradarv2 ]]

repo_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
runner_user=airradarv2-runner
runner_home=/var/lib/airradarv2-runner
runner_dir="$runner_home/actions-runner"
if ! id "$runner_user" >/dev/null 2>&1; then
  useradd --system --home-dir "$runner_home" --create-home --shell /usr/sbin/nologin "$runner_user"
fi
install -d -o "$runner_user" -g "$runner_user" -m 0750 "$runner_dir" "$runner_home/_work"

metadata="$(mktemp)"
archive="$(mktemp --suffix=.tar.gz)"
trap 'rm -f -- "$metadata" "$archive"' EXIT
curl -fsSL --retry 3 -H 'Accept: application/vnd.github+json' \
  https://api.github.com/repos/actions/runner/releases/latest -o "$metadata"
version="$(jq -r '.tag_name | ltrimstr("v")' "$metadata")"
asset="actions-runner-linux-x64-${version}.tar.gz"
url="$(jq -r --arg n "$asset" '.assets[] | select(.name == $n) | .browser_download_url' "$metadata")"
digest="$(jq -r --arg n "$asset" '.assets[] | select(.name == $n) | .digest // empty' "$metadata")"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ && "$url" == https://github.com/actions/runner/releases/download/* ]]
curl -fsSL --retry 3 "$url" -o "$archive"
if [[ "$digest" == sha256:* ]]; then
  test "sha256:$(sha256sum "$archive" | awk '{print $1}')" = "$digest"
fi
tar -tzf "$archive" >/dev/null
runuser -u "$runner_user" -- tar -xzf "$archive" -C "$runner_dir"
chown -R "$runner_user:$runner_user" "$runner_dir"
if [[ -e "$runner_dir/.runner" ]]; then
  runuser -u "$runner_user" -- "$runner_dir/config.sh" remove --token "$RUNNER_REGISTRATION_TOKEN" || true
fi
runuser -u "$runner_user" -- "$runner_dir/config.sh" \
  --url https://github.com/Boym323/AirRadarV2 \
  --token "$RUNNER_REGISTRATION_TOKEN" --name lo-dev-airradarv2 \
  --labels airradarv2-dev --work _work --unattended --replace
unset RUNNER_REGISTRATION_TOKEN

install -o root -g root -m 0755 "$repo_dir/deploy/runner-deploy.sh" /usr/local/libexec/airradarv2-runner-deploy.sh
install -o root -g root -m 0644 "$repo_dir/deploy/archive_validation.py" /usr/local/libexec/archive_validation.py
install -o root -g root -m 0755 "$repo_dir/deploy/main-dev-auto-pull.py" /usr/local/libexec/airradarv2-main-deploy-pull.py
install -o root -g root -m 0755 "$repo_dir/deploy/install-dev-bundle.sh" /usr/local/libexec/airradarv2-main-deploy-install.sh
install -o root -g root -m 0644 "$repo_dir/deploy/systemd/airradarv2-runner.service" /etc/systemd/system/airradarv2-runner.service
install -o root -g root -m 0440 "$repo_dir/deploy/systemd/airradarv2-runner.sudoers" /etc/sudoers.d/airradarv2-runner
install -d -o root -g root -m 0755 \
  /etc/systemd/system/airradarv2-g02-auto.service.d \
  /etc/systemd/system/airradarv2-g02-live-auto.service.d
install -o root -g root -m 0644 "$repo_dir/deploy/systemd/airradarv2-g02-lock.conf" \
  /etc/systemd/system/airradarv2-g02-auto.service.d/lock.conf
install -o root -g root -m 0644 "$repo_dir/deploy/systemd/airradarv2-g02-live-lock.conf" \
  /etc/systemd/system/airradarv2-g02-live-auto.service.d/lock.conf
install -o root -g root -m 0660 /dev/null /run/lock/airradarv2-main-deploy.lock
chown root:airradarv2 /run/lock/airradarv2-main-deploy.lock
visudo -cf /etc/sudoers.d/airradarv2-runner
install -d -o "$runner_user" -g "$runner_user" -m 0700 /var/lib/airradarv2-runner/incoming
systemctl daemon-reload
systemctl enable --now airradarv2-runner.service
systemctl --no-pager --full status airradarv2-runner.service
