#!/usr/bin/env bash
set -Eeuo pipefail
if [[ "${G04_TEST_ENGINE_MODE:-ok}" == stale ]]; then observed_at=0; else observed_at="$(date +%s)"; fi
echo "G-04 history observed_at=$observed_at pending=0 inflight=0 eligible=1 coalesced=0 committed=1 dropped_capacity=0 dropped_stale=0 dropped_writer=0 sql_attempts=1 committed_batches=1 failed_batches=0 sql_timeouts=0 connection=Up"
if [[ -n "${G04_TEST_PID_MARKER:-}" ]]; then printf '%s' "$BASHPID" > "$G04_TEST_PID_MARKER"; fi
trap 'if [[ -n "${G04_TEST_TERM_MARKER:-}" ]]; then printf terminated > "$G04_TEST_TERM_MARKER"; fi; exit 143' TERM INT
sleep "${G04_TEST_ENGINE_SECONDS:-3}"
