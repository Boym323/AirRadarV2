#!/usr/bin/env python3
"""Vyhodnotí 30minutový G-04 canary bez přístupu k tajemstvím.

Vstup diagnostics je JSONL nebo journal export s řádky obsahujícími
"G-04 history pending=...". metrics je JSON s provozními důkazy dodanými
odděleným monitorigem (SSE, CPU/RAM, WAL a DB readback).
"""
import argparse
import json
import re
import sys
from pathlib import Path

FIELDS = ("pending", "inflight", "eligible", "committed", "dropped_capacity",
          "dropped_stale", "dropped_writer", "sql_timeouts")
PATTERN = re.compile(r"G-04 history\s+(.+)$")


def read_diagnostics(path):
    snapshots = []
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        match = PATTERN.search(line)
        if not match:
            try:
                item = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(item, dict) and "pending" in item:
                snapshots.append(item)
            continue
        item = {}
        for token in match.group(1).split():
            if "=" not in token:
                continue
            key, value = token.split("=", 1)
            value = value.rstrip(",")
            if key in FIELDS:
                item[key] = int(value)
            elif key == "connection":
                item[key] = value.lower().strip('"')
        if all(key in item for key in ("pending", "inflight")):
            snapshots.append(item)
    return snapshots


def evaluate(snapshots, metrics):
    result = {"status": "FAIL", "checks": {}, "observations": {}}
    if not snapshots:
        result["checks"]["diagnostics_present"] = False
        return result
    last = snapshots[-1]
    drops = sum(last.get(key, 0) for key in
                ("dropped_capacity", "dropped_stale", "dropped_writer"))
    eligible = last.get("eligible", 0)
    loss = 100.0 * drops / max(1, eligible + drops)
    pending = [item.get("pending", 0) for item in snapshots]
    result["observations"].update({"snapshots": len(snapshots),
                                    "loss_percent": round(loss, 4),
                                    "max_pending": max(pending),
                                    "last": last})
    result["checks"] = {
        "diagnostics_present": True,
        "pending_bound": max(pending) <= 2048,
        "drained": last.get("pending") == 0 and last.get("inflight") == 0,
        "connection_up": last.get("connection") in (None, "up"),
        "sql_timeouts_zero": last.get("sql_timeouts", 0) == 0,
        "candidate_loss_le_1_percent": loss <= 1.0,
        "no_persistent_queue_growth": len(pending) < 3 or pending[-1] <= max(pending[-3:]),
        "duration_1800_seconds": metrics.get("duration_seconds", 0) >= 1800,
        "actual_db_write": metrics.get("actual_db_write") is True,
        "g03_sse_healthy": metrics.get("g03_sse_healthy") is True,
        "host_metrics_present": all(metrics.get(key) is True for key in
                                      ("cpu_ram_recorded", "wal_recorded")),
    }
    result["status"] = "PASS" if all(result["checks"].values()) else "FAIL"
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--diagnostics", required=True)
    parser.add_argument("--metrics", required=True)
    args = parser.parse_args()
    result = evaluate(read_diagnostics(args.diagnostics),
                      json.loads(Path(args.metrics).read_text(encoding="utf-8")))
    print(json.dumps(result, ensure_ascii=False, indent=2, sort_keys=True))
    return 0 if result["status"] == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
