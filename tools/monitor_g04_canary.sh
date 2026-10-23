#!/usr/bin/env bash
set -Eeuo pipefail

# G-04.10/J09: read-only dohled canary. Wrapper vlastní pouze svůj podřízený
# G-04 proces; G-03 nikdy nerestartuje ani nezastavuje.
[[ ${1:-} == "--" && $# -ge 2 ]] || { echo "Použití: $0 -- <engine> [argumenty]" >&2; exit 64; }
shift

interval="${AIRRADAR_G04_MONITOR_INTERVAL_SEC:-15}"
free_floor="${AIRRADAR_G04_MIN_FREE_BYTES:-2147483648}"
max_wal_rate="${AIRRADAR_G04_MAX_WAL_GROWTH_BYTES_PER_SEC:-10485760}"
max_cpu="${AIRRADAR_G04_MAX_DB_CPU_LOAD_PCT:-80}"
max_ram="${AIRRADAR_G04_MAX_DB_RAM_USED_PCT:-90}"
data_path="${AIRRADAR_G04_DB_DATA_PATH:-/var/lib/postgresql}"
wal_path="${AIRRADAR_G04_DB_WAL_PATH:-$data_path}"
ssh_target="${AIRRADAR_G04_DB_MONITOR_SSH_TARGET:-}"
health_url="${AIRRADAR_G04_G03_HEALTH_URL:-http://127.0.0.1:4101/api/v1/health}"
env_file="${AIRRADAR_G04_ENV_FILE:-/etc/airradar-v2/g04-canary.env}"

[[ "$interval" =~ ^[1-9][0-9]*$ && "$free_floor" =~ ^[0-9]+$ && "$max_wal_rate" =~ ^[0-9]+$ ]] || { echo 'G-04 monitor: neplatný interval/místo/WAL limit' >&2; exit 78; }
[[ "$max_cpu" =~ ^[0-9]+$ && "$max_ram" =~ ^[0-9]+$ && -n "$ssh_target" ]] || { echo 'G-04 monitor: chybí schválený monitorovací přístup nebo CPU/RAM limit' >&2; exit 78; }
[[ -f "$env_file" ]] && [[ "$(stat -c '%u:%g:%a' "$env_file" 2>/dev/null)" == "0:0:600" ]] || { echo 'G-04 monitor: EnvironmentFile není root:root 0600 (fail-closed)' >&2; exit 78; }

scratch="${TMPDIR:-/tmp}/airradar-g04-monitor.$$"
mkdir -m 0700 "$scratch"
diag="$scratch/diagnostics.log"
trap 'rm -f "$diag"; rmdir "$scratch" 2>/dev/null || true' EXIT

fail_closed() {
  echo "G-04 fail-closed: $1" >&2
  if [[ -n "${child_pid:-}" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill -TERM "$child_pid" 2>/dev/null || true
    for _ in 1 2 3 4 5; do kill -0 "$child_pid" 2>/dev/null || break; sleep 1; done
    kill -KILL "$child_pid" 2>/dev/null || true
  fi
  wait "${child_pid:-}" 2>/dev/null || true
  exit 1
}

read_diag() {
  local line value key
  line="$(grep 'G-04 history' "$diag" | tail -n 1 || true)"
  [[ -n "$line" ]] || return 1
  for key in pending inflight connection sql_timeouts; do
    value="$(sed -n "s/.*$key=\\([^ ]*\\).*/\\1/p" <<<"$line")"
    case "$key" in
      connection) [[ "$value" == "Up" || "$value" == "up" ]] || return 1 ;;
      sql_timeouts) [[ "$value" == 0 ]] || return 1 ;;
      *) [[ "$value" =~ ^[0-9]+$ ]] || return 1 ;;
    esac
  done
}

check_g03() {
  local health
  health="$(curl --silent --show-error --fail --max-time 2 "$health_url")" || return 1
  [[ "$health" == *'"status":"live"'* && "$health" == *'"stage":"G-03"'* && "$health" == *'"live_ingest":true'* ]]
}

read_db_metrics() {
  # Pouze SELECT; heslo zůstává v prostředí psql a nikdy se neloguje.
  PGHOST="${AIRRADAR_V2_DEV_DB_HOST:?}" PGPORT="${AIRRADAR_V2_DEV_DB_PORT:-5432}" \
  PGUSER="${AIRRADAR_V2_DEV_DB_USER:?}" PGPASSWORD="${AIRRADAR_V2_DEV_DB_PASSWORD:?}" \
  psql -X -At -d airradar_v2_dev -v ON_ERROR_STOP=1 -w -c \
    "SELECT coalesce((SELECT wal_bytes FROM pg_stat_wal),0), coalesce((SELECT sum(size) FROM pg_ls_waldir()),0);" 2>/dev/null
}

read_host_metrics() {
  ssh -T -o BatchMode=yes -o ConnectTimeout=3 "$ssh_target" \
    "awk 'NR==1 {print \"load1=\"\$1}' /proc/loadavg; nproc | awk '{print \"cpus=\"\$1}'; free -b | awk '/^Mem:/ {print \"ram_total=\"\$2; print \"ram_available=\"\$7}'; df -Pk --output=avail '$data_path' '$wal_path' | awk 'NR>1 {print \"free_kib=\"\$1}'" 2>/dev/null
}

child_pid=""
"$@" > >(tee -a "$diag") 2> >(tee -a "$diag" >&2) &
child_pid=$!
previous_wal=""
previous_at=""
while kill -0 "$child_pid" 2>/dev/null; do
  sleep "$interval"
  kill -0 "$child_pid" 2>/dev/null || break
  read_diag || fail_closed 'chybí aktuální/úplná G-04 diagnostika'
  check_g03 || fail_closed 'G-03 není live nebo health není dostupný'
  db_metrics="$(read_db_metrics)" || fail_closed 'nedostupné SQL read-only metriky'
  IFS='|' read -r wal_bytes wal_dir_bytes <<< "$db_metrics"
  [[ "$wal_bytes" =~ ^[0-9]+$ && "$wal_dir_bytes" =~ ^[0-9]+$ ]] || fail_closed 'neplatná WAL metrika'
  host_metrics="$(read_host_metrics)" || fail_closed 'nedostupné aktuální metriky DB LXC'
  load1="$(sed -n 's/^load1=//p' <<<"$host_metrics")"; cpus="$(sed -n 's/^cpus=//p' <<<"$host_metrics")"
  ram_total="$(sed -n 's/^ram_total=//p' <<<"$host_metrics")"; ram_available="$(sed -n 's/^ram_available=//p' <<<"$host_metrics")"
  free_kib="$(sed -n 's/^free_kib=//p' <<<"$host_metrics" | tail -n 1)"
  [[ "$load1" =~ ^[0-9]+([.][0-9]+)?$ && "$cpus" =~ ^[1-9][0-9]*$ && "$ram_total" =~ ^[1-9][0-9]*$ && "$ram_available" =~ ^[0-9]+$ && "$free_kib" =~ ^[0-9]+$ ]] || fail_closed 'neúplná/nesmyslná metrika DB LXC'
  ram_used_pct="$(awk -v t="$ram_total" -v a="$ram_available" 'BEGIN { printf "%d", ((t-a)*100)/t }')"
  cpu_load_pct="$(awk -v l="$load1" -v c="$cpus" 'BEGIN { printf "%d", (l*100)/c }')"
  (( free_kib * 1024 >= free_floor )) || fail_closed 'volné místo DB LXC pod 2 GiB'
  (( ram_used_pct <= max_ram )) || fail_closed 'RAM DB LXC nad limitem'
  (( cpu_load_pct <= max_cpu )) || fail_closed 'CPU/load DB LXC nad limitem'
  now="$(date +%s)"
  if [[ -n "$previous_wal" ]]; then
    elapsed=$((now - previous_at)); (( elapsed > 0 && wal_bytes >= previous_wal )) || fail_closed 'neplatný nebo klesající WAL údaj'
    wal_rate=$(( (wal_bytes - previous_wal) / elapsed )); (( wal_rate <= max_wal_rate )) || fail_closed 'nebezpečný růst WAL'
  fi
  previous_wal="$wal_bytes"; previous_at="$now"
  echo "G-04 monitor: PASS free_bytes=$((free_kib * 1024)) cpu_load_pct=$cpu_load_pct ram_used_pct=$ram_used_pct wal_bytes=$wal_bytes"
done
child_status=0
wait "$child_pid" || child_status=$?
(( child_status == 0 )) || exit "$child_status"
echo 'G-04 monitor: canary skončil bez selhání monitoru' >&2
