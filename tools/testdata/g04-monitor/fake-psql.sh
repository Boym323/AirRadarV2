#!/usr/bin/env bash
set -Eeuo pipefail
if [[ "${G04_TEST_PSQL_MODE:-ok}" == fail ]]; then exit 124; fi
if [[ -n "${G04_TEST_PSQL_SEQUENCE_FILE:-}" ]]; then
  value="$(head -n 1 "$G04_TEST_PSQL_SEQUENCE_FILE")"
  sed -i '1d' "$G04_TEST_PSQL_SEQUENCE_FILE"
  printf '%s\n' "$value"
else
  printf '%s\n' "${G04_TEST_WAL_BYTES:-1000}"
fi
