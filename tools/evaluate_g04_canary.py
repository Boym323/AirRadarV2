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
from datetime import datetime, timezone
from pathlib import Path

FIELDS = ("pending", "inflight", "eligible", "committed", "dropped_capacity",
          "dropped_stale", "dropped_writer", "sql_timeouts", "observed_at")
COUNTERS = ("pending", "inflight", "eligible", "committed", "dropped_capacity",
            "dropped_stale", "dropped_writer", "sql_timeouts")
PATTERN = re.compile(r"G-04 history\s+(.+)$")


def _timestamp(value):
    """Vrátí monotónní/epoch čas v sekundách; bez něj není snapshot důkazem."""
    if isinstance(value, (int, float)) and value == value:
        return float(value)
    if isinstance(value, str):
        try:
            return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(
                timezone.utc).timestamp()
        except ValueError:
            return None
    return None


def _normalise_connection(value):
    value = str(value).strip().lower().strip('"')
    value = value.removeprefix("some(").removesuffix(")")
    return {"up": "up", "down": "down", "unknown": "unknown", "none": "unknown"}.get(value, value)


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
                if "observed_at" not in item:
                    item["observed_at"] = item.get("timestamp", item.get("monotonic_seconds"))
                if "connection" in item:
                    item["connection"] = _normalise_connection(item["connection"])
                snapshots.append(item)
            continue
        item = {}
        for token in match.group(1).split():
            if "=" not in token:
                continue
            key, value = token.split("=", 1)
            value = value.rstrip(",")
            if key in FIELDS:
                if key == "observed_at":
                    item[key] = _timestamp(value)
                    continue
                item[key] = int(value)
            elif key == "connection":
                item[key] = _normalise_connection(value)
        if all(key in item for key in ("pending", "inflight")):
            snapshots.append(item)
    return snapshots


def evaluate(snapshots, metrics):
    result = {"status": "FAIL", "checks": {}, "observations": {}}
    if not snapshots:
        result["checks"]["diagnostics_present"] = False
        return result
    required = set(COUNTERS) | {"connection", "observed_at"}
    complete = all(required.issubset(item) and
                   all(isinstance(item.get(key), int) and item[key] >= 0 for key in COUNTERS) and
                   _timestamp(item.get("observed_at")) is not None
                   for item in snapshots)
    timestamps = [_timestamp(item.get("observed_at")) for item in snapshots]
    ordered = complete and all(a < b for a, b in zip(timestamps, timestamps[1:]))
    cumulative = ("eligible", "committed", "dropped_capacity", "dropped_stale",
                  "dropped_writer", "sql_timeouts")
    counters_monotonic = complete and all(
        all(previous[key] <= current[key] for key in cumulative)
        for previous, current in zip(snapshots, snapshots[1:]))
    last = snapshots[-1]
    drops = sum(last.get(key, 0) for key in
                ("dropped_capacity", "dropped_stale", "dropped_writer"))
    eligible = last.get("eligible", 0)
    # Rust zvyšuje eligible při vložení do bounded fronty a každý drop je
    # podmnožina těchto kandidátů. Přičítat dropy znovu by jmenovatel nafouklo.
    loss = 100.0 * drops / eligible if eligible else 100.0
    pending = [item.get("pending", 0) for item in snapshots]
    trailing_growth = (len(pending) >= 4 and
                       all(a < b for a, b in zip(pending[-4:], pending[-3:])))
    run_elapsed = _timestamp(metrics.get("finished_at"))
    run_started = _timestamp(metrics.get("started_at"))
    timestamp_span = timestamps[-1] - timestamps[0] if ordered and len(timestamps) > 1 else 0
    wall_span = run_elapsed - run_started if run_started is not None and run_elapsed is not None else 0
    final_balance = (last.get("committed", 0) + drops + last.get("pending", 0) +
                     last.get("inflight", 0) == eligible)
    result["observations"].update({"snapshots": len(snapshots),
                                    "loss_percent": round(loss, 4),
                                    "max_pending": max(pending),
                                    "diagnostic_span_seconds": round(timestamp_span, 3),
                                    "run_span_seconds": round(wall_span, 3),
                                    "last": last})
    result["checks"] = {
        "diagnostics_present": True,
        "diagnostics_complete_and_ordered": complete and ordered and counters_monotonic,
        "pending_bound": max(pending) <= 2048,
        "drained": last.get("pending") == 0 and last.get("inflight") == 0,
        "counter_balance": final_balance,
        "connection_up": all(item.get("connection") == "up" for item in snapshots),
        "sql_timeouts_zero": all(item.get("sql_timeouts") == 0 for item in snapshots),
        "candidate_loss_le_1_percent": loss <= 1.0,
        "no_persistent_queue_growth": len(pending) >= 4 and not trailing_growth,
        "duration_1800_seconds": wall_span >= 1800 and timestamp_span >= 1800,
        "actual_db_readback": (metrics.get("db_readback_verified") is True and
                                isinstance(metrics.get("db_readback_rows"), int) and
                                metrics["db_readback_rows"] > 0),
        "g03_sse_healthy": (metrics.get("g03_sse_healthy") is True and
                             isinstance(metrics.get("g03_sse_heartbeat_count"), int) and
                             metrics["g03_sse_heartbeat_count"] >= 2),
        "host_metrics_present": all(metrics.get(key) is True for key in
                                      ("cpu_ram_recorded", "wal_recorded", "db_free_space_recorded")),
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
