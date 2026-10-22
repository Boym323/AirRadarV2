"""Unit testy omezeni G-02 pull agenta bez internetu a bez LXC."""
import importlib.util
import hashlib
import json
from unittest.mock import patch
import io
from pathlib import Path
import tarfile
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    "g02_auto_pull", Path(__file__).resolve().parent.parent / "deploy/g02-auto-pull.py"
)
agent = importlib.util.module_from_spec(spec)
spec.loader.exec_module(agent)


def make_archive(sha, payload=b"binary-data", symlink=False):
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode="w:gz") as tar:
        for name, body in [("./VERSION", (sha + "\n").encode()),
                           ("./airradarv2-engine", payload)]:
            member = tarfile.TarInfo(name)
            if symlink and "engine" in name:
                member.type = tarfile.SYMTYPE
                member.linkname = "/etc/passwd"
                tar.addfile(member)
            else:
                member.size = len(body)
                tar.addfile(member, io.BytesIO(body))
    buffer.seek(0)
    return buffer


class AgentTests(unittest.TestCase):
    def test_only_pinned_prerelease(self):
        sha = "a" * 40
        filename = "airradarv2-dev-linux-amd64-" + sha[:12] + ".tar.gz"
        valid = {"tag_name": "g02-dev-" + sha, "draft": False,
                 "prerelease": True, "target_commitish": sha,
                 "assets": [{"name": filename}, {"name": filename + ".sha256"}]}
        self.assertEqual(agent.candidate([valid])[0], sha)
        for change in [{"target_commitish": "b" * 40}, {"draft": True},
                       {"prerelease": False}, {"tag_name": "release-latest"},
                       {"assets": []}]:
            self.assertIsNone(agent.candidate([{**valid, **change}]))

    def test_reject_symlinks_and_wrong_version(self):
        sha = "b" * 40
        with tempfile.TemporaryDirectory() as root:
            output = Path(root) / "binary"
            agent.extract_binary(make_archive(sha), output, sha)
            self.assertEqual(output.read_bytes(), b"binary-data")
            with self.assertRaises(ValueError):
                agent.extract_binary(make_archive(sha, symlink=True), output, sha)
            with self.assertRaises(ValueError):
                agent.extract_binary(make_archive("c" * 40), output, sha)

    def test_end_to_end_without_network(self):
        sha = "d" * 40
        filename = "airradarv2-dev-linux-amd64-" + sha[:12] + ".tar.gz"
        executable = (
            "#!/bin/sh\n"
            "case \"$1\" in\n"
            "  --g02-replay) echo 'G-02 fixture replay';;\n"
            "  --g02-benchmark)\n"
            "    for n in 1 2 4; do\n"
            "      printf 'G-02 bench %sx: ok\n' \"$n\"\n"
            "      printf 'G-02 5000-target stress %sx: ok\n' \"$n\"\n"
            "    done;;\n"
            "  *) exit 2;;\n"
            "esac\n"
        ).encode()
        bundle = make_archive(sha, executable).getvalue()
        digest = hashlib.sha256(bundle).hexdigest()
        checksum = (digest + "  " + filename + "\n").encode()
        base = "https://github.com/" + agent.REPO + "/releases/download/g02-dev-" + sha + "/"
        assets = [
            {"name": filename, "browser_download_url": base + filename,
             "digest": "sha256:" + digest},
            {"name": filename + ".sha256", "browser_download_url": base + filename + ".sha256",
             "digest": "sha256:" + hashlib.sha256(checksum).hexdigest()},
        ]
        release = {"tag_name": "g02-dev-" + sha, "target_commitish": sha,
                   "prerelease": True, "draft": False, "assets": assets}

        def mocked_download(url, limit):
            if url.endswith("/releases?per_page=25"):
                return json.dumps([release]).encode()
            if "/git/ref/tags/" in url:
                return json.dumps({"object": {"type": "commit", "sha": sha}}).encode()
            if url.endswith(filename):
                return bundle
            if url.endswith(filename + ".sha256"):
                return checksum
            raise AssertionError(url)

        with tempfile.TemporaryDirectory() as root:
            with patch.object(agent, "PRIVATE", Path(root) / "private"), \
                 patch.object(agent, "PUBLIC", Path(root) / "public"), \
                 patch.object(agent, "download", side_effect=mocked_download) as mocked:
                self.assertEqual(agent.main(), 0)
                result = json.loads((Path(root) / "public/status.json").read_text())
                self.assertEqual(result["status"], "PASS_SYNTHETIC_LXC")
                self.assertEqual(result["benchmark"], "PASS")
                self.assertEqual(result["sha"], sha)
                self.assertEqual(agent.main(), 0)
                self.assertEqual(mocked.call_count, 5)


if __name__ == "__main__":
    unittest.main()
