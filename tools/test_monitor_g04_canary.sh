#!/usr/bin/env bash
set -Eeuo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
monitor="$root/tools/monitor_g04_canary.sh"
fixture="$root/tools/testdata/g04-monitor"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
printf '%s\n' '# testovací prázdný env; skutečná tajemství se zde nepoužívají' > "$tmp/env"
chmod 0600 "$tmp/env"
export AIRRADAR_G04_TEST_MODE=1 AIRRADAR_G04_ENV_FILE="$tmp/env"
export AIRRADAR_G04_DB_MONITOR_SSH_TARGET=airradar-g04-monitor@test-db-lxc
export AIRRADAR_V2_DEV_DB_HOST=127.0.0.1 AIRRADAR_V2_DEV_DB_PORT=5432
export AIRRADAR_V2_DEV_DB_USER=airradar_v2_dev_app AIRRADAR_V2_DEV_DB_PASSWORD=fixture-only
export AIRRADAR_G04_MONITOR_INTERVAL_SEC=1 AIRRADAR_G04_SSH_TIMEOUT_SEC=1
export AIRRADAR_G04_CURL_BIN="$fixture/fake-curl.sh" AIRRADAR_G04_PSQL_BIN="$fixture/fake-psql.sh"
export AIRRADAR_G04_SSH_BIN="$fixture/fake-ssh.sh" AIRRADAR_G04_TIMEOUT_BIN=timeout

run_expect_failure() {
  local name="$1"; shift; local marker="$tmp/$name.term"
  local pid_marker="$tmp/$name.pid"
  set +e
  G04_TEST_TERM_MARKER="$marker" G04_TEST_PID_MARKER="$pid_marker" G04_TEST_ENGINE_SECONDS=20 "$@" "$monitor" -- "$fixture/fake-engine.sh"
  local rc=$?
  set -e
  [[ "$rc" -ne 0 ]] || { echo "FAIL: $name nefailoval" >&2; exit 1; }
  [[ -s "$pid_marker" ]] || { echo "FAIL: $name child G-04 se nespustil" >&2; exit 1; }
  if kill -0 "$(cat "$pid_marker")" 2>/dev/null; then echo "FAIL: $name child G-04 stále běží" >&2; exit 1; fi
}

G04_TEST_ENGINE_MODE=ok G04_TEST_SSH_MODE=ok G04_TEST_PSQL_MODE=ok "$monitor" -- "$fixture/fake-engine.sh" >/dev/null
run_expect_failure remote-vypadek env G04_TEST_SSH_MODE=fail
run_expect_failure ssh-timeout env G04_TEST_SSH_MODE=timeout
run_expect_failure stale-diagnostika env G04_TEST_ENGINE_MODE=stale
run_expect_failure g03-degradace env G04_TEST_CURL_MODE=degraded
run_expect_failure nizke-disky env G04_TEST_SSH_MODE=low_disk
run_expect_failure sql-vypadek env G04_TEST_PSQL_MODE=fail
printf '%s\n' 1000 100000000 > "$tmp/wal.sequence"
run_expect_failure wal-rust env G04_TEST_PSQL_SEQUENCE_FILE="$tmp/wal.sequence"

if env AIRRADAR_G04_DB_MONITOR_SSH_TARGET= "$monitor" -- "$fixture/fake-engine.sh" >/dev/null 2>&1; then
  echo 'FAIL: chybějící SSH cíl nebyl fail-closed' >&2; exit 1
fi
bash -n "$monitor" "$root/deploy/g04-db-monitor-readonly.sh"
grep -q 'CPUQuota=50%' "$root/deploy/systemd/airradarv2-g04-canary.service"
printf '%s\n' 'PASS: funkční G-04 monitor testy PASS, výpadky/stáří/limity fail-closed a child G-04 zastaven.'
