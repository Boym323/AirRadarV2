#!/usr/bin/env bash
set -Eeuo pipefail

# Instalovat pouze po schválení DBA na PostgreSQL LXC jako root:root 0755.
# Cesty musí odpovídat skutečnému DB/WAL filesystemu; účet SSH nemá právo je měnit.
readonly DATA_PATH=/var/lib/postgresql
readonly WAL_PATH=/var/lib/postgresql

awk 'NR == 1 { print "load1=" $1 }' /proc/loadavg
nproc | awk '{ print "cpus=" $1 }'
free -b | awk '/^Mem:/ { print "ram_total=" $2; print "ram_available=" $7 }'
df -Pk "$DATA_PATH" | awk 'NR == 2 { print "free_data_kib=" $4 }'
df -Pk "$WAL_PATH" | awk 'NR == 2 { print "free_wal_kib=" $4 }'
