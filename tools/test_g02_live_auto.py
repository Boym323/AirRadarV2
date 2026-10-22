"""Offline tests for GitHub release request selection and duplicate safety."""
import datetime as dt
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "g02_live_poll", Path(__file__).resolve().parent.parent / "deploy/g02-live-auto.py"
)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
SHA = "a" * 40
NOW = dt.datetime(2026, 10, 22, 14, 0, tzinfo=dt.timezone.utc)


def request(id, sha=SHA, published="2026-10-22T13:59:00Z", **overrides):
    return {**{"tag_name": f"g02-live-{sha}-{id}", "prerelease": True,
               "target_commitish": sha, "draft": False, "assets": [],
               "author": {"login": "github-actions[bot]"},
               "published_at": published}, **overrides}


class TestRequest(unittest.TestCase):
    def test_select_unsorted_and_ignore_replays(self):
        releases = [request(2, published="2026-10-22T13:58:00Z"),
                    request(3), request(1, published="2026-10-22T13:57:00Z")]
        self.assertEqual(mod.select_request(releases, SHA, set(), NOW)[1], 3)
        self.assertEqual(mod.select_request(releases, SHA, {3}, NOW)[1], 2)
        self.assertIsNone(mod.select_request(releases, SHA, {1, 2, 3}, NOW))

    def test_reject_forged_or_stale_requests(self):
        examples = [
            request(1, sha="b" * 40),
            request(2, author={"login": "attacker"}),
            request(3, target_commitish="b" * 40),
            request(4, prerelease=False),
            request(5, draft=True),
            request(6, published_at="2026-10-22T12:00:00Z"),
            request(7, assets=[{"name": "cmd.sh"}]),
            request(8, published_at="not-a-date"),
            request(9, tag_name="g02-live-a-untrusted"),
        ]
        for item in examples:
            with self.subTest(item=item["tag_name"]):
                self.assertIsNone(mod.select_request([item], SHA, set(), NOW))


if __name__ == "__main__":
    unittest.main()
