#!/usr/bin/env python3
"""G-02.11: verify DEV LXC public acceptance report strictly by expected SHA."""
import argparse
import json
from pathlib import Path
import re
import sys

SHA = re.compile(r"^[0-9a-f]{40}$")
SOAK = re.compile(
    r"^G-02 isolated-loopback soak PASS:"
    r".*\bduration_ms=(\d+)\b"
    r".*\btcp_connects=(\d+)\b"
    r".*\btcp_eofs=(\d+)\b"
    r".*\bbeast_frames=(\d+)\b"
    r".*\bjson_ok=(\d+)\b"
    r".*\bjson_fail=(\d+)\b"
    r".*\bqueue_drops=(\d+)\b"
    r".*\bp95_receive_us=(\d+)\b"
    r".*\brss_peak_kib=(\d+)\b"
    r".*\btracks_after_ttl=(\d+)\b"
)


def validate(report, expected_sha):
    if not SHA.fullmatch(expected_sha):
        return False, "Neplatné očekávané SHA"
    if report.get("sha") != expected_sha:
        return False, "LXC zatím netestovalo přesný aktuální commit"
    if report.get("stage") != "G-02.11":
        return False, "Nesprávná testovací etapa"
    if report.get("status") != "PASS_SYNTHETIC_LXC":
        return False, "Offline test neprošel"
    if report.get("replay") != "PASS" or report.get("benchmark") != "PASS":
        return False, "Chybí PASS replay/benchmark"
    visible_cpu_count = report.get("visibleCpuCount")
    if not isinstance(visible_cpu_count, int) or isinstance(visible_cpu_count, bool) or visible_cpu_count < 1:
        return False, "LXC neposkytlo kladný počet viditelných CPU"
    log = report.get("benchmarkLog", "")
    for scale in ("1", "2", "4"):
        if (f"G-02 bench {scale}x:" not in log
                or f"G-02 5000-target stress {scale}x:" not in log):
            return False, "Chybí některý syntetický zátěžový scénář"
    soak = next((SOAK.search(line) for line in log.splitlines()
                 if line.startswith("G-02 isolated-loopback soak PASS:")), None)
    if soak is None:
        return False, "Chybí předepsaná tříminutová TCP/JSON zkouška"
    duration, connects, eofs, frames, json_ok, json_fail, drops, p95, rss, ttl = (
        int(v) for v in soak.groups()
    )
    if not (180000 <= duration <= 300000):
        return False, "Délka loopback soak není 180 sekund"
    if not (connects >= 4 and eofs >= 3 and frames >= 4):
        return False, "Neproběhl dostatečný BEAST reconnect"
    if not (json_ok >= 2 and json_fail >= 1):
        return False, "Neprokázán JSON výpadek a obnova"
    if drops != 0 or ttl != 0 or rss <= 0:
        return False, "Selhal invariant fronty, TTL nebo měření RSS"
    if p95 >= 1_000_000:
        return False, "P95 ingest převyšuje 1 sekundu"
    return True, (
        f"PASS: DEV LXC SHA={expected_sha} "
        f"duration_ms={duration} TCP={connects}/{eofs} "
        f"BEAST={frames} JSON={json_ok}/{json_fail} "
        f"queue_drops={drops} p95_us={p95} rss_peak_kib={rss} ttl={ttl}"
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--file", type=Path, required=True)
    parser.add_argument("--sha", required=True)
    args = parser.parse_args()
    try:
        report = json.loads(args.file.read_text(encoding="utf-8"))
        ok, message = validate(report, args.sha)
        print(message)
        return 0 if ok else 1
    except (OSError, ValueError) as exc:
        print("Neplatný report G-02:", str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
