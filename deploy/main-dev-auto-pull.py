#!/usr/bin/env python3
"""G-13 DEV: root-owned, SHA-bound pull-only deployment of tested main releases.

Only GitHub Actions bot releases built by g02-dev-candidate.yml on the exact
current main commit are eligible. No SSH, webhook, DB or receiver mutation.
"""
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tarfile
import tempfile
import urllib.error
import urllib.parse
import urllib.request

REPOSITORY = "Boym323/AirRadarV2"
API = "https://api.github.com/repos/" + REPOSITORY
STATE = Path("/var/lib/airradar-v2/main-deploy")
PUBLIC = Path("/var/cache/airradarv2-main-deploy/status.json")
CURRENT_VERSION = Path("/opt/airradar-v2/current/VERSION")
INSTALLER = Path("/usr/local/libexec/airradarv2-main-deploy-install.sh")
SHA = re.compile(r"[0-9a-f]{40}")
BOT = "github-actions[bot]"
BUNDLE_MAX = 150 * 1024 * 1024
HEADERS = {
    "User-Agent": "AirRadarV2-main-DEV-auto-deploy",
    "Accept": "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
}


def fetch(url: str, max_size: int) -> bytes:
    if not url.startswith("https://"):
        raise ValueError("HTTP destination is not TLS")
    request = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(request, timeout=15) as response:
        final = urllib.parse.urlparse(response.url)
        if final.scheme != "https" or final.hostname not in (
            "api.github.com", "github.com", "release-assets.githubusercontent.com",
            "objects.githubusercontent.com",
        ):
            raise ValueError("Unexpected download redirect host")
        payload = response.read(max_size + 1)
    if len(payload) > max_size:
        raise ValueError("Download exceeds size limit")
    return payload


def github_json(endpoint: str) -> dict:
    return json.loads(fetch(API + endpoint, 2_000_000))


def target_main_sha() -> str:
    ref = github_json("/git/ref/heads/main")
    sha = ref.get("object", {}).get("sha", "")
    if not SHA.fullmatch(sha):
        raise ValueError("GitHub main does not provide immutable SHA")
    return sha


def eligible_release(release: dict, main_sha: str) -> tuple:
    """Never accept branch names, stale releases, 3rd-party authors or renamed assets."""
    if not SHA.fullmatch(main_sha):
        raise ValueError("Invalid main SHA")
    tag = "g02-dev-" + main_sha
    name = "airradarv2-dev-linux-amd64-" + main_sha[:12] + ".tar.gz"
    if (
        release.get("tag_name") != tag
        or release.get("target_commitish") != main_sha
        or release.get("draft") is not False
        or release.get("prerelease") is not True
        or release.get("author", {}).get("login") != BOT
    ):
        raise ValueError("Release not from approved immutable main workflow")
    assets = {item.get("name"): item for item in release.get("assets", [])}
    if len(assets) != len(release.get("assets", [])) or not {name, name + ".sha256"} <= assets.keys():
        raise ValueError("Release assets missing or duplicated")
    for asset_name in (name, name + ".sha256"):
        asset = assets[asset_name]
        prefix = f"https://github.com/{REPOSITORY}/releases/download/{tag}/{asset_name}"
        if asset.get("browser_download_url") != prefix:
            raise ValueError("Release download URL mismatch")
        if asset.get("state") not in (None, "uploaded"):
            raise ValueError("Release asset not published")
    return tag, name, assets[name], assets[name + ".sha256"]


def sha256_asset(asset: dict, limit: int) -> tuple:
    payload = fetch(asset["browser_download_url"], limit)
    digest = hashlib.sha256(payload).hexdigest()
    if asset.get("digest") not in (None, "sha256:" + digest):
        raise ValueError("GitHub asset digest mismatch")
    return payload, digest


def check_archive(data: bytes, sha: str) -> None:
    """Validate all tar members before executing a root-owned installer."""
    import io
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        members = archive.getmembers()
        if len(members) > 20000:
            raise ValueError("Too many archive members")
        total, found = 0, {}
        for entry in members:
            name = entry.name.removeprefix("./").rstrip("/")
            if not name and entry.isdir():
                continue
            path = name.split("/")
            if (
                not name or name.startswith("/") or "\\" in name
                or any(part in ("", ".", "..") for part in path)
                or not (name in ("VERSION", "airradarv2-engine", "web") or name.startswith("web/"))
                or not (entry.isfile() or entry.isdir()) or name in found
            ):
                raise ValueError("Unsafe archive member")
            found[name] = entry
            total += entry.size if entry.isfile() else 0
        if total > 300 * 1024 * 1024:
            raise ValueError("Archive uncompressed size too large")
        if not {"VERSION", "airradarv2-engine", "web/index.html"} <= found.keys():
            raise ValueError("Missing required files")
        if not all(found[item].isfile() for item in ("VERSION", "airradarv2-engine", "web/index.html")):
            raise ValueError("Nonregular critical archive member")
        source = archive.extractfile(found["VERSION"])
        if source is None or source.read(256).decode("ascii", "strict").strip() != sha:
            raise ValueError("Archive VERSION != main commit")


def current_sha() -> str:
    try:
        current = CURRENT_VERSION.read_text(encoding="ascii").strip()
    except OSError:
        return ""
    return current if SHA.fullmatch(current) else ""


def write_status(status: str, sha: str = "", reason: str = "") -> None:
    PUBLIC.parent.mkdir(parents=True, exist_ok=True)
    body = {
        "stage": "AirRadarV2 DEV main deployment",
        "status": status,
        "deployedSha": current_sha() or None,
        "candidateSha": sha or None,
        "checkedAtUtc": dt.datetime.now(dt.timezone.utc).isoformat(),
    }
    if reason:
        body["reason"] = reason
    stage = PUBLIC.with_suffix(".json.tmp")
    stage.write_text(json.dumps(body, sort_keys=True, ensure_ascii=False) + "\n", encoding="utf-8")
    os.chmod(stage, 0o644)
    stage.replace(PUBLIC)


def g02_acceptance_active() -> bool:
    # Both offline acceptance and the hourly physical BEAST soak own the 1-vCPU box.
    for service in ("airradarv2-g02-live-auto.service",
                    "airradarv2-g02-auto.service"):
        r = subprocess.run(
            ["systemctl", "show", "--value", "-p", "ActiveState", service],
            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True,
            timeout=10, check=False,
        )
        if r.stdout.strip() in {"active", "activating", "reloading"}:
            return True
    return False


def preflight() -> None:
    if os.geteuid() != 0:
        raise ValueError("Run only through root-owned systemd DEV service")
    os_release = Path("/etc/os-release").read_text(encoding="utf-8")
    if 'ID=debian' not in os_release or 'VERSION_ID="13"' not in os_release:
        raise ValueError("Not the approved Debian 13 DEV LXC")
    if os.uname().machine != "x86_64" or os.uname().nodename != "lo-dev-airradarv2":
        raise ValueError("DEV LXC hostname or architecture mismatch")
    if not INSTALLER.is_file():
        raise ValueError("Missing pinned root-owned installer")
    for path in (INSTALLER, Path("/usr/local/libexec/airradarv2-main-deploy-pull.py")):
        st = path.stat()
        if st.st_uid != 0 or st.st_mode & 0o022:
            raise ValueError("Agent/installer is not immutable for unprivileged users")
    if not CURRENT_VERSION.is_file():
        raise ValueError("No bootstrapped DEV application")
    real = CURRENT_VERSION.parent.resolve()
    if not real.is_relative_to("/opt/airradar-v2/releases"):
        raise ValueError("Current release escaped isolated DEV directory")


def run() -> int:
    preflight()
    STATE.mkdir(parents=True, exist_ok=True, mode=0o700)
    os.chmod(STATE, 0o700)
    main_sha = target_main_sha()
    if current_sha() == main_sha:
        write_status("DEPLOYED", main_sha)
        print("DEV already at current main:", main_sha)
        return 0
    blocked = STATE / "blocked.json"
    if blocked.is_file():
        try:
            if json.loads(blocked.read_text()).get("sha") == main_sha:
                write_status("ROLLBACK_BLOCKED", main_sha, "Previous health/install failure; new SHA needed")
                return 0
        except (OSError, ValueError):
            pass
    if g02_acceptance_active():
        write_status("POSTPONED_G02_ACCEPTANCE", main_sha, "LIVE acceptance takes priority")
        return 0
    tag = "g02-dev-" + main_sha
    try:
        release = github_json("/releases/tags/" + tag)
    except urllib.error.HTTPError as exc:
        if exc.code == 404:
            write_status("WAITING_FOR_CI", main_sha, "No tested release for current main")
            return 0
        raise
    selected_tag, filename, bundle, checksum = eligible_release(release, main_sha)
    ref = github_json("/git/ref/tags/" + selected_tag)
    if ref.get("object", {}).get("type") != "commit" or ref["object"]["sha"] != main_sha:
        raise ValueError("Tag does not point directly to exact main commit")
    archive, digest = sha256_asset(bundle, BUNDLE_MAX)
    signature, _ = sha256_asset(checksum, 512)
    expected = digest + "  " + filename
    if signature.decode("ascii", "strict").strip() != expected:
        raise ValueError("Published SHA-256 manifest mismatch")
    check_archive(archive, main_sha)
    # Never overwrite a newer commit that landed while downloading an older one.
    if target_main_sha() != main_sha:
        write_status("NEWER_MAIN_PENDING", main_sha)
        return 0
    if g02_acceptance_active():
        write_status("POSTPONED_G02_SOAK", main_sha)
        return 0
    # Directory and files remain root-owned; the running web user cannot replace executable data.
    with tempfile.TemporaryDirectory(prefix="bundle-", dir=STATE) as staging:
        location = Path(staging) / filename
        location.write_bytes(archive)
        location.with_name(filename + ".sha256").write_text(expected + "\n", encoding="ascii")
        os.chmod(location, 0o600)
        os.chmod(location.with_name(filename + ".sha256"), 0o600)
        env = {"PATH": "/usr/sbin:/usr/bin:/sbin:/bin",
               "LANG": "C.UTF-8",
               "AIRRADAR_V2_AUTO_DEPLOY": "1",
               "AIRRADAR_EXPECTED_SHA": main_sha}
        process = subprocess.Popen(
            ["/usr/bin/bash", str(INSTALLER), str(location)],
            cwd=staging, env=env,
        )
        try:
            returncode = process.wait(timeout=180)
        except subprocess.TimeoutExpired:
            # Bash handles SIGTERM and executes its EXIT rollback; avoid
            # subprocess.run(timeout=...), which would SIGKILL without cleanup.
            process.send_signal(signal.SIGTERM)
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
            raise ValueError("DEV installer deadline hit; verify rollback")
    if returncode != 0 or current_sha() != main_sha:
        blocked.write_text(json.dumps({"sha": main_sha}) + "\n", encoding="utf-8")
        os.chmod(blocked, 0o600)
        write_status("ROLLED_BACK", main_sha, "Installer/health failed, same SHA blocked")
        return 1
    if blocked.exists():
        blocked.unlink()
    write_status("DEPLOYED", main_sha)
    print("PASS: V2 DEV auto-deployed approved main", main_sha)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(run())
    except (OSError, ValueError, KeyError, TypeError, subprocess.TimeoutExpired,
            urllib.error.URLError, json.JSONDecodeError) as exc:
        print("DEV auto-deploy rejected safely:", type(exc).__name__, str(exc), file=sys.stderr)
        try:
            write_status("ERROR_NO_DEPLOY", reason=type(exc).__name__)
        except OSError:
            pass
        sys.exit(1)
