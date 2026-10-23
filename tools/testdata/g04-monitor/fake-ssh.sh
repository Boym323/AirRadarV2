#!/usr/bin/env bash
set -Eeuo pipefail
case "${G04_TEST_SSH_MODE:-ok}" in
  fail) exit 255 ;;
  timeout) sleep 10; exit 0 ;;
  low_disk) printf '%s\n' 'load1=0.1' 'cpus=4' 'ram_total=1000000' 'ram_available=900000' 'free_data_kib=1024' 'free_wal_kib=4194304' ;;
  *) printf '%s\n' 'load1=0.1' 'cpus=4' 'ram_total=1000000' 'ram_available=900000' 'free_data_kib=4194304' 'free_wal_kib=4194304' ;;
esac
