"""G-02.11: unit test evidence verifier without any network."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "g02_verify", Path(__file__).resolve().parent / "verify_g02_public_status.py"
)
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)
SHA = "a" * 40
LOG = "\n".join(
    [f"G-02 bench {n}x: ok\nG-02 5000-target stress {n}x: ok"
     for n in (1, 2, 4)]
    + ["G-02 isolated-loopback soak PASS: duration_ms=180010 tcp_connects=65 "
       "tcp_eofs=65 beast_frames=65 json_ok=178 json_fail=3 queue_drops=0 "
       "p95_receive_us=66 rss_start_kib=3000 rss_peak_kib=5240 "
       "tracks_after_ttl=0 receiver=127.0.0.1(no V1/DB)"]
)
BASE = {"sha": SHA, "stage": "G-02.11", "status": "PASS_SYNTHETIC_LXC",
        "replay": "PASS", "benchmark": "PASS", "visibleCpuCount": 1,
        "benchmarkLog": LOG}


class TestVerifier(unittest.TestCase):
    def test_valid_report(self):
        passed, msg = verify.validate(BASE, SHA)
        self.assertTrue(passed, msg)

    def test_reject_wrong_sha_and_short_test(self):
        self.assertFalse(verify.validate(BASE, "b" * 40)[0])
        self.assertFalse(verify.validate({**BASE, "benchmarkLog":
            LOG.replace("duration_ms=180010", "duration_ms=5000")}, SHA)[0])

    def test_reject_failed_reconnect_and_queue(self):
        self.assertFalse(verify.validate({**BASE, "benchmarkLog":
            LOG.replace("tcp_eofs=65", "tcp_eofs=0")}, SHA)[0])
        self.assertFalse(verify.validate({**BASE, "benchmarkLog":
            LOG.replace("queue_drops=0", "queue_drops=1")}, SHA)[0])

    def test_accept_report_with_host_visible_cpu_count(self):
        passed, msg = verify.validate({**BASE, "visibleCpuCount": 4}, SHA)
        self.assertTrue(passed, msg)


if __name__ == "__main__":
    unittest.main()
