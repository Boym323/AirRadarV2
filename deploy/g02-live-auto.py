#!/usr/bin/env python3
"""G-02.11: read GitHub-owned request releases; fixed read-only receiver soak."""
import datetime as dt
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys

AGENT_PATH = Path("/usr/local/libexec/airradarv2-g02-pull.py")
SOAK = Path("/usr/local/libexec/airradarv2-g02-live-soak.py")
STATUS = Path("/var/cache/airradarv2-g02-public/status.json")
PRIVATE = Path("/var/lib/airradar-v2/g02-auto")
REQUEST = re.compile(r"g02-live-([0-9a-f]{40})-([1-9][0-9]{0,18})")
BOT = "github-actions[bot]"


def select_request(releases, expected_sha, used_ids, now=None):
    now = now or dt.datetime.now(dt.timezone.utc)
    acceptable = []
    for release in releases:
        match = REQUEST.fullmatch(release.get("tag_name", ""))
        if not match or match.group(1) != expected_sha:
            continue
        rid = int(match.group(2))
        if rid in used_ids or release.get("draft") or not release.get("prerelease"):
            continue
        if (release.get("target_commitish") != expected_sha
                or release.get("author", {}).get("login") != BOT
                or release.get("assets")):
            continue
        try:
            published = dt.datetime.fromisoformat(
                release["published_at"].replace("Z", "+00:00")
            )
            age = (now - published).total_seconds()
        except (KeyError, TypeError, ValueError):
            continue
        if published.tzinfo is None or not -60 <= age <= 2700:
            continue
        acceptable.append((published, rid, match.group(0)))
    return max(acceptable, default=None)


def result_update(agent, request_id, sha, status, **extra):
    current = json.loads(STATUS.read_text(encoding="utf-8"))
    if current.get("sha") != sha:
        return
    current["liveSoak"] = {
        "sha": sha,
        "requestRunId": request_id,
        "status": status,
        **extra,
    }
    agent.atomically_write(STATUS, current)


def main():
    if os.geteuid() == 0:
        raise ValueError("Live agent must run as non-root user")
    receiver = os.environ.get("AIRRADAR_V2_G02_RECEIVER", "")
    port = int(os.environ.get("AIRRADAR_V2_G02_PORT", "30005"))
    # Dynamically load installed verifier, only to enforce the receiver allowlist.
    live_spec = importlib.util.spec_from_file_location("live_soak", SOAK)
    live = importlib.util.module_from_spec(live_spec)
    live_spec.loader.exec_module(live)
    receiver = live.validate_receiver(receiver, port)
    agent_spec = importlib.util.spec_from_file_location("g02_agent", AGENT_PATH)
    agent = importlib.util.module_from_spec(agent_spec)
    agent_spec.loader.exec_module(agent)
    status = json.loads(STATUS.read_text(encoding="utf-8"))
    if (status.get("status") != "PASS_SYNTHETIC_LXC"
            or not re.fullmatch("[0-9a-f]{40}", status.get("sha", ""))):
        print("G-02: no verified offline SHA; cannot start live")
        return 0
    sha = status["sha"]
    candidates = json.loads(agent.download(agent.API + "/releases?per_page=100", 5_000_000))
    state = PRIVATE / "live-requests"
    state.mkdir(mode=0o700, parents=True, exist_ok=True)
    used = {int(p.stem) for p in state.glob("*.json") if p.stem.isdecimal()}
    found = select_request(candidates, sha, used)
    if found is None:
        print("G-02: no new signed GitHub live request for current SHA")
        return 0
    _, rid, tag = found
    # Extra immutable tag check before accepting a privileged 60-minute action.
    ref = json.loads(agent.download(agent.API + "/git/ref/tags/" + tag, 100_000))
    if ref.get("object", {}).get("type") != "commit" or ref["object"]["sha"] != sha:
        raise ValueError("Live request tag is not attached to approved commit")
    marker = state / (str(rid) + ".json")
    agent.atomically_write(marker, {"runId": rid, "sha": sha, "accepted": True})
    result_update(agent, rid, sha, "RUNNING")
    try:
        completed = subprocess.run(
            [sys.executable, str(SOAK), "--sha", sha, "--request-id", str(rid),
             "--receiver", receiver, "--port", str(port), "--seconds", "3600"],
            cwd=str(PRIVATE),
            env={"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"},
            timeout=3800, check=False,
        )
        outcome = json.loads(STATUS.read_text(encoding="utf-8"))
        live_result = outcome.get("liveSoak", {})
        if (live_result.get("requestRunId") != rid
                or live_result.get("status") == "RUNNING"):
            result_update(agent, rid, sha, "FAIL", reason="Live run produced no validated report")
        return 0 if completed.returncode == 0 else 1
    except (subprocess.TimeoutExpired, OSError, ValueError) as exc:
        result_update(agent, rid, sha, "FAIL", reason="Timeout or test process failure")
        print("G-02 live:", type(exc).__name__, file=sys.stderr)
        return 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, TypeError) as exc:
        print("G-02: no safe live test:", str(exc), file=sys.stderr)
        sys.exit(1)
