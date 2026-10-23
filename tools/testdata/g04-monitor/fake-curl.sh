#!/usr/bin/env bash
set -Eeuo pipefail
if [[ "${G04_TEST_CURL_MODE:-live}" == degraded ]]; then
  printf '%s\n' '{"status":"degraded","stage":"G-03","live_ingest":false}'
else
  printf '%s\n' '{"status":"live","stage":"G-03","live_ingest":true}'
fi
