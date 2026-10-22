#!/usr/bin/env python3
"""G-02.11 offline benchmark: GitHub release pull-only, no production deploy."""
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
import urllib.request

REPO = "Boym323/AirRadarV2"
API = "https://api.github.com/repos/" + REPO
PATTERN = re.compile(r"g02-dev-([0-9a-f]{40})")
PRIVATE = Path("/var/lib/airradar-v2/g02-auto")
PUBLIC = Path("/var/cache/airradarv2-g02-public")
HEADERS = {
    "User-Agent": "AirRadarV2-G02-DEV",
    "Accept": "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
}


def download(url, limit):
    if not url.startswith("https://"):
        raise ValueError("Povoleno pouze HTTPS")
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req, timeout=35) as response:
        if not response.url.startswith("https://"):
            raise ValueError("Nezabezpecene presmerovani")
        data = response.read(limit + 1)
    if len(data) > limit:
        raise ValueError("Soubor presahuje bezpecny limit")
    return data


def candidate(releases):
    for rel in releases:
        match = PATTERN.fullmatch(rel.get("tag_name", ""))
        if not match or rel.get("draft") or not rel.get("prerelease"):
            continue
        sha = match.group(1)
        if rel.get("target_commitish") != sha:
            continue
        filename = "airradarv2-dev-linux-amd64-" + sha[:12] + ".tar.gz"
        assets = {a["name"]: a for a in rel.get("assets", [])}
        if filename in assets and filename + ".sha256" in assets:
            return sha, rel["tag_name"], assets[filename], assets[filename + ".sha256"]
    return None


def fetch_asset(asset, limit):
    url = asset.get("browser_download_url", "")
    prefix = "https://github.com/" + REPO + "/releases/download/"
    if not url.startswith(prefix):
        raise ValueError("Neocekavany puvod binarky")
    content = download(url, limit)
    digest = hashlib.sha256(content).hexdigest()
    if asset.get("digest") not in (None, "sha256:" + digest):
        raise ValueError("GitHub asset SHA-256 neodpovida")
    return content, digest


def extract_binary(archive, dest, sha):
    # No tar.extractall: trusted checksum AND only known regular members.
    with tarfile.open(fileobj=archive, mode="r:gz") as tf:
        members = {}
        for info in tf:
            name = info.name.removeprefix("./")
            if name in ("VERSION", "airradarv2-engine"):
                if name in members or not info.isfile():
                    raise ValueError("Neplatny clen tar archivu")
                members[name] = info
        if set(members) != {"VERSION", "airradarv2-engine"}:
            raise ValueError("Chybi VERSION nebo binarka")
        if members["airradarv2-engine"].size > 100 * 1024 * 1024:
            raise ValueError("Prilis velka binarka")
        f = tf.extractfile(members["VERSION"])
        if f is None or f.read(128).decode("ascii", "strict").strip() != sha:
            raise ValueError("VERSION se neshoduje se SHA releasu")
        source = tf.extractfile(members["airradarv2-engine"])
        if source is None:
            raise ValueError("Binarka neni regular file")
        data = source.read(100 * 1024 * 1024 + 1)
        if len(data) != members["airradarv2-engine"].size:
            raise ValueError("Velikost binarky nesouhlasi")
        dest.write_bytes(data)
        dest.chmod(0o700)


def atomically_write(path, data):
    temp = path.with_name(path.name + ".tmp")
    temp.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    os.chmod(temp, 0o644)
    temp.replace(path)


def run(binary, flag, seconds):
    return subprocess.run(
        [str(binary), flag], cwd=str(binary.parent),
        env={"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"},
        timeout=seconds, capture_output=True, text=True, check=False
    )


def main():
    PRIVATE.mkdir(parents=True, exist_ok=True)
    PUBLIC.mkdir(parents=True, exist_ok=True)
    selected = candidate(json.loads(download(API + "/releases?per_page=25", 2_000_000)))
    if selected is None:
        print("G-02: zatim neni k dispozici overeny DEV kandidat")
        return 0
    sha, tag, bundle, checksum = selected
    status_path = PUBLIC / "status.json"
    if status_path.exists():
        try:
            if json.loads(status_path.read_text()).get("sha") == sha:
                print("G-02: SHA uz bylo vyhodnoceno", sha)
                return 0
        except (OSError, ValueError):
            pass
    ref = json.loads(download(API + "/git/ref/tags/" + tag, 100_000))
    if ref.get("object", {}).get("type") != "commit" or ref["object"].get("sha") != sha:
        raise ValueError("Tag neukazuje na ocekavany commit")
    with tempfile.TemporaryDirectory(prefix="accept-", dir=PRIVATE) as tmp:
        dest = Path(tmp) / "airradarv2-engine"
        compressed, digest = fetch_asset(bundle, 150 * 1024 * 1024)
        sha_text, _ = fetch_asset(checksum, 256)
        expected = digest + "  airradarv2-dev-linux-amd64-" + sha[:12] + ".tar.gz"
        if sha_text.decode("ascii").strip() != expected:
            raise ValueError("SHA256 manifest neodpovida archivu")
        import io
        extract_binary(io.BytesIO(compressed), dest, sha)
        result = {
            "stage": "G-02.11", "sha": sha,
            "testedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
            "environment": "DEV LXC; offline replay + synteticky benchmark",
            "visibleCpuCount": os.cpu_count(),
            "cpuQuota": Path("/sys/fs/cgroup/cpu.max").read_text().strip()
                if Path("/sys/fs/cgroup/cpu.max").exists() else "NEOVĚŘENO",
            "replay": "NEOVĚŘENO", "benchmark": "NEOVĚŘENO", "status": "FAIL",
        }
        try:
            replay = run(dest, "--g02-replay", 90)
            result["replay"] = "PASS" if replay.returncode == 0 else "FAIL"
            result["replayLog"] = (replay.stdout + replay.stderr)[-4000:]
            if replay.returncode == 0:
                bench = run(dest, "--g02-benchmark", 300)
                lines = bench.stdout
                all_cases = all(
                    "G-02 bench " + n + "x:" in lines
                    and "G-02 5000-target stress " + n + "x:" in lines
                    for n in ("1", "2", "4")
                )
                result["benchmark"] = "PASS" if bench.returncode == 0 and all_cases else "FAIL"
                result["benchmarkLog"] = (bench.stdout + bench.stderr)[-16000:]
                if result["benchmark"] == "PASS":
                    result["status"] = "PASS_SYNTHETIC_LXC"
        except subprocess.TimeoutExpired:
            result["error"] = "Prekrocen casovy limit"
        atomically_write(PRIVATE / "last-result.json", result)
        atomically_write(status_path, result)
        print("G-02:", result["status"], sha)
        return 0 if result["status"] == "PASS_SYNTHETIC_LXC" else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError) as exc:
        print("G-02: stahovani nebo overeni selhalo; zadny zasah do sluzby: " + str(exc), file=sys.stderr)
        sys.exit(1)
