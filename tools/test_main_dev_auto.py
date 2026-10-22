"""G-13 DEV: offline guard tests; no external GitHub/LAN or systemd required."""
import importlib.util
import io
from pathlib import Path
import tarfile
import unittest

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("auto_dev", ROOT / "deploy/main-dev-auto-pull.py")
agent = importlib.util.module_from_spec(spec)
spec.loader.exec_module(agent)
SHA = "a" * 40
TAG = "g02-dev-" + SHA
NAME = "airradarv2-dev-linux-amd64-" + SHA[:12] + ".tar.gz"


def asset(name):
    return {
        "name": name,
        "state": "uploaded",
        "browser_download_url": f"https://github.com/Boym323/AirRadarV2/releases/download/{TAG}/{name}",
    }


def release(**overrides):
    return {
        "tag_name": TAG,
        "target_commitish": SHA,
        "draft": False,
        "prerelease": True,
        "author": {"login": "github-actions[bot]"},
        "assets": [asset(NAME), asset(NAME + ".sha256")],
        **overrides,
    }


def archive(extra=(), corrupt_version=False):
    result = io.BytesIO()
    with tarfile.open(fileobj=result, mode="w:gz") as tf:
        for path, content in (
            ("VERSION", ("b" * 40 if corrupt_version else SHA).encode()),
            ("airradarv2-engine", b"RUST_BINARY_MOCK"),
            ("web/index.html", b'<div id="root"></div>'),
            *extra,
        ):
            metadata = tarfile.TarInfo(path)
            if content is None:
                metadata.type = tarfile.SYMTYPE
                metadata.linkname = "../../etc/passwd"
            else:
                metadata.size = len(content)
            tf.addfile(metadata, io.BytesIO(content) if content is not None else None)
    return result.getvalue()


class MainDeploySafety(unittest.TestCase):
    def test_only_main_commit_releases_from_workflow_bot(self):
        selected = agent.eligible_release(release(), SHA)
        self.assertEqual(selected[0], TAG)
        self.assertEqual(selected[1], NAME)
        rejected = (
            {"tag_name": "g02-dev-" + "b" * 40},
            {"target_commitish": "main"},
            {"draft": True},
            {"prerelease": False},
            {"author": {"login": "attacker"}},
            {"assets": [asset(NAME)]},
            {"assets": [asset(NAME), asset(NAME), asset(NAME + ".sha256")]},
            {"assets": [{"name": NAME, "browser_download_url": "https://example.org/evil"},
                        asset(NAME + ".sha256")]},
        )
        for case in rejected:
            with self.subTest(case=case):
                with self.assertRaises(ValueError):
                    agent.eligible_release(release(**case), SHA)

    def test_archive_allowlist_version_and_symlink(self):
        agent.check_archive(archive(), SHA)
        for bad in (
            archive(corrupt_version=True),
            archive(extra=(("../etc/passwd", b"bad"),)),
            archive(extra=(("web/untrusted", None),)),
            archive(extra=(("VERSION", b"duplicate"),)),
        ):
            with self.subTest(payload=bad[:30]):
                with self.assertRaises(ValueError):
                    agent.check_archive(bad, SHA)

    def test_install_script_hardening_and_timer(self):
        install = (ROOT / "deploy/install-dev-bundle.sh").read_text()
        poll = (ROOT / "deploy/install-main-dev-auto.sh").read_text()
        for keyword in (
            "AIRRADAR_EXPECTED_SHA", "AIRRADAR_V2_AUTO_DEPLOY",
            "trap 'rollback", "sha256sum -c", "nginx -t",
            "systemctl restart airradarv2-engine",
            "health_tmp", "tarfile", "web/index.html",
        ):
            self.assertIn(keyword, install)
        for keyword in (
            "TimeoutStartSec=300", "Type=oneshot", "NoNewPrivileges=yes",
            "ProtectSystem=strict", "ProtectHome=yes", "MemoryMax=768M",
            "OnUnitInactiveSec=5min", "User=root",
            "G13_SOURCE_REF", "airradarv2-main-deploy.timer",
        ):
            self.assertIn(keyword, poll)
        self.assertNotIn("ssh ", poll)
        self.assertNotIn("docker ", poll)


if __name__ == "__main__":
    unittest.main()
