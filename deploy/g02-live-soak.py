#!/usr/bin/env python3
"""G-02.11: opt-in read-only 60 min real BEAST output soak on dedicated V2 DEV LXC.

Runs the already vetted immutable G-02 engine in --g02-live mode, never the
installed G-00 service. No SQL, transmitter, API, external feeder or V1 writes.
"""
import argparse
import datetime as dt
import importlib.util
import io
import ipaddress
import json
import os
from pathlib import Path
import re
import signal
import socket
import subprocess
import sys
import time

SHA_PATTERN = re.compile(r"^[0-9a-f]{40}$")
TRACE = re.compile(
    r"G-02 RAM tracks=(\d+) updates=(\d+) duplicates=(\d+) "
    r"conflicts=(\d+) capacity_drops=(\d+) channel_drops=(\d+) "
    r"preferred=(Offline|Beast|Json)"
)
ROOT = Path("/var/lib/airradar-v2/g02-auto")
STATUS = Path("/var/cache/airradarv2-g02-public/status.json")
AGENT = Path("/usr/local/libexec/airradarv2-g02-pull.py")
LIMIT_KIB = 256 * 1024


def validate_receiver(host, port):
    try:
        ip = ipaddress.IPv4Address(host)
    except ipaddress.AddressValueError as exc:
        raise ValueError("Vyžadována explicitní privátní IPv4 adresa") from exc
    if not (ip.is_private and not ip.is_loopback and not ip.is_link_local):
        raise ValueError("Přijímač musí mít privátní IPv4 adresu v LAN")
    if port != 30005:
        raise ValueError("Povolen je výhradně BEAST OUTPUT port 30005 (nikoli INPUT 30004)")
    return str(ip)


def evaluate(lines, *, duration, peak_rss, cpu_percent, ended_early):
    rows = []
    for line in lines:
        match = TRACE.search(line)
        if match:
            values = [int(value) for value in match.groups()[:6]]
            rows.append((*values, match.group(7)))
    if not rows:
        return {"status": "FAIL", "reason": "Chybí G-02 diagnostika běžícího ingestu"}
    first, last = rows[0], rows[-1]
    frames = (last[1] - first[1]) + (last[2] - first[2]) + (last[3] - first[3])
    active_seconds = sum(row[6] == "Beast" for row in rows)
    peak_tracks = max(row[0] for row in rows)
    max_capacity = max(row[4] for row in rows)
    max_channel = max(row[5] for row in rows)
    result = {
        "durationSec": round(duration, 1),
        "samples": len(rows),
        "activeBeastSamples": active_seconds,
        "receivedObservationsAtLeast": max(0, frames),
        "updated": last[1],
        "duplicates": last[2],
        "conflicts": last[3],
        "peakTracks": peak_tracks,
        "capacityDrops": max_capacity,
        "channelDrops": max_channel,
        "peakRssKiB": peak_rss,
        "averageCpuPercent": round(cpu_percent, 2) if cpu_percent is not None else None,
    }
    if ended_early or duration < 3595:
        result.update(status="FAIL", reason="Test nedokončil plných 60 minut")
    elif max_channel or max_capacity:
        result.update(status="FAIL", reason="Zjištěné ztráty ve frontě nebo limitu tracků")
    elif peak_rss <= 0 or peak_rss >= LIMIT_KIB:
        result.update(status="FAIL", reason="RSS nelze změřit nebo překročila 256 MiB")
    elif frames <= 0 or active_seconds <= 0 or peak_tracks <= 0:
        result.update(status="INCONCLUSIVE", reason="Nepotvrzen skutečný přijímaný DF17 provoz")
    else:
        result.update(status="PASS_LIVE_BEAST_60MIN")
    return result


def proc_stats(pid):
    try:
        status = Path(f"/proc/{pid}/status").read_text(encoding="utf-8")
        rss = next(int(row.split()[1]) for row in status.splitlines()
                   if row.startswith("VmRSS:"))
        stat = Path(f"/proc/{pid}/stat").read_text(encoding="ascii")
        fields = stat.rsplit(") ", 1)[1].split()
        ticks = int(fields[11]) + int(fields[12])
        return rss, ticks
    except (OSError, ValueError, IndexError, StopIteration):
        return None, None


def fetch_engine(agent, sha, dest):
    if not SHA_PATTERN.fullmatch(sha):
        raise ValueError("Neplatné SHA")
    release = json.loads(agent.download(
        f"{agent.API}/releases/tags/g02-dev-{sha}", 300_000))
    if (release.get("draft") or not release.get("prerelease")
            or release.get("target_commitish") != sha
            or release.get("tag_name") != f"g02-dev-{sha}"):
        raise ValueError("Vydání není immutable G-02 DEV kandidát pro očekávané SHA")
    ref = json.loads(agent.download(f"{agent.API}/git/ref/tags/g02-dev-{sha}", 100_000))
    if ref.get("object") != {"type": "commit", "sha": sha}:
        raise ValueError("Tag neukazuje na požadovaný commit")
    filename = f"airradarv2-dev-linux-amd64-{sha[:12]}.tar.gz"
    assets = {asset["name"]: asset for asset in release["assets"]}
    data, digest = agent.fetch_asset(assets[filename], 150 * 1024 * 1024)
    manifest, _ = agent.fetch_asset(assets[filename + ".sha256"], 256)
    if manifest.decode("ascii").strip() != f"{digest}  {filename}":
        raise ValueError("Manifest SHA-256 nesouhlasí")
    agent.extract_binary(io.BytesIO(data), dest, sha)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--receiver", required=True)
    parser.add_argument("--port", type=int, default=30005)
    parser.add_argument("--sha", required=True)
    parser.add_argument("--seconds", type=int, default=3600)
    args = parser.parse_args()
    receiver = validate_receiver(args.receiver, args.port)
    if args.seconds != 3600:
        raise ValueError("Přejímací test má pevnou délku 3 600 sekund")
    if os.geteuid() == 0:
        raise ValueError("Spouštěj pouze pod neprivilegovaným účtem airradarv2")
    if not SHA_PATTERN.fullmatch(args.sha):
        raise ValueError("Neplatné SHA")
    initial = json.loads(STATUS.read_text(encoding="utf-8"))
    if initial.get("sha") != args.sha or initial.get("status") != "PASS_SYNTHETIC_LXC":
        raise ValueError("Nejprve musí PASS offline test přesného SHA v g02-status.json")
    # Abort before any download or test unless the receiver already accepts
    # a read-only Beast TCP output connection.
    with socket.create_connection((receiver, args.port), timeout=4):
        pass

    spec = importlib.util.spec_from_file_location("g02_agent", AGENT)
    agent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(agent)
    workdir = ROOT / "real-soak"
    workdir.mkdir(mode=0o700, parents=True, exist_ok=True)
    os.chmod(workdir, 0o700)
    binary = workdir / "airradarv2-engine"
    fetch_engine(agent, args.sha, binary)
    logpath = workdir / "real-beast.log"
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    start = time.monotonic()
    peak = 0
    initial_ticks = None
    last_ticks = None
    ended_early = False
    try:
        with logpath.open("w", encoding="utf-8") as log:
            env = {
                "PATH": "/usr/bin:/bin",
                "LANG": "C.UTF-8",
                "AIRRADAR_V2_BEAST_ADDR": f"{receiver}:{args.port}",
                "AIRRADAR_V2_G02_TRACE": "1",
            }
            process = subprocess.Popen(
                [str(binary), "--g02-live"],
                cwd=workdir, env=env, stdin=subprocess.DEVNULL,
                stdout=log, stderr=subprocess.STDOUT,
                start_new_session=True,
            )
            while time.monotonic() - start < args.seconds:
                rss, ticks = proc_stats(process.pid)
                if rss is not None:
                    peak = max(peak, rss)
                if ticks is not None:
                    if initial_ticks is None:
                        initial_ticks = ticks
                    last_ticks = ticks
                if process.poll() is not None:
                    ended_early = True
                    break
                time.sleep(5)
            if not ended_early:
                process.send_signal(signal.SIGTERM)
            try:
                process.wait(timeout=8)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
        elapsed = time.monotonic() - start
        if ended_early:
            print(f"Rust ingest ended early: rc={process.returncode}", file=sys.stderr)
        cpu_percent = None
        if initial_ticks is not None and last_ticks is not None and elapsed > 0:
            cpu_percent = (
                (last_ticks - initial_ticks) / os.sysconf("SC_CLK_TCK")
                / elapsed * 100.0
            )
        lines = logpath.read_text(encoding="utf-8", errors="replace").splitlines()
        report = evaluate(lines, duration=elapsed, peak_rss=peak,
                          cpu_percent=cpu_percent, ended_early=ended_early)
        report.update(
            sha=args.sha, startedAtUtc=started,
            endedAtUtc=dt.datetime.now(dt.timezone.utc).isoformat(),
            source="read-only Beast TCP output / LAN (adresa neveřejná)",
            mode="real BEAST / no JSON, DB, HTTP, API, SSE or V1 changes",
            latencyNote="P95 živé ingest latence zatím není v tomto režimu instrumentováno",
        )
        # Logs (incl. per-second diagnostic counters) remain private.
        agent.atomically_write(workdir / "live-result.json", report)
        latest = json.loads(STATUS.read_text(encoding="utf-8"))
        if latest.get("sha") != args.sha:
            raise ValueError("Veřejný status mezitím změnil SHA; výsledek je jen privátně")
        latest["liveSoak"] = report
        agent.atomically_write(STATUS, latest)
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return 0 if report["status"] == "PASS_LIVE_BEAST_60MIN" else 1
    finally:
        # A single immutable temporary acceptance binary; no service deployment.
        binary.unlink(missing_ok=True)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, ConnectionError) as exc:
        print(f"G-02 LIVE soak FAIL: {exc}", file=sys.stderr)
        sys.exit(1)
