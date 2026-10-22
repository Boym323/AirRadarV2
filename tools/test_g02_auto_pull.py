"""Unit testy omezeni G-02 pull agenta bez internetu a bez LXC."""
import importlib.util
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


if __name__ == "__main__":
    unittest.main()
