"""G-02.11: pure offline verification of receiver preflight and live evidence."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "g02_live", Path(__file__).resolve().parent.parent / "deploy/g02-live-soak.py"
)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


def logs(source="Beast", updates=2, channel=0, capacity=0, duplicates=1):
    return [
        "G-02 RAM tracks=1 updates=1 duplicates=0 conflicts=0 "
        "capacity_drops=0 channel_drops=0 preferred=Beast",
        f"G-02 RAM tracks=2 updates={updates} duplicates={duplicates} conflicts=0 "
        f"capacity_drops={capacity} channel_drops={channel} preferred={source}",
    ]


class SoakTests(unittest.TestCase):
    def test_read_only_beast_output_only(self):
        self.assertEqual(mod.validate_receiver("192.168.1.50", 30005),
                         "192.168.1.50")
        for host, port in [
            ("192.168.1.50", 30004), ("8.8.8.8", 30005),
            ("127.0.0.1", 30005), ("receiver.local", 30005),
        ]:
            with self.subTest(host=host, port=port):
                with self.assertRaises(ValueError):
                    mod.validate_receiver(host, port)

    def test_pass_requires_real_frames_and_low_ram(self):
        result = mod.evaluate(logs(), duration=3600.5, peak_rss=5500,
                              cpu_percent=1.7, ended_early=False)
        self.assertEqual(result["status"], "PASS_LIVE_BEAST_60MIN")
        self.assertEqual(result["receivedObservationsAtLeast"], 2)

    def test_missing_frames_short_run_and_drops_rejected(self):
        cases = [
            (logs(source="Offline"), 3600, 5000, False, "PASS_LIVE_BEAST_60MIN"),
            (logs(updates=1, duplicates=0, source="Offline"), 3600, 5000, False, "INCONCLUSIVE"),
            (logs(channel=1), 3600, 5000, False, "FAIL"),
            (logs(capacity=1), 3600, 5000, False, "FAIL"),
            (logs(), 5, 5000, False, "FAIL"),
            (logs(), 3600, 5000, True, "FAIL"),
            (logs(), 3600, 300000, False, "FAIL"),
        ]
        for text, duration, rss, early, expected in cases:
            result = mod.evaluate(text, duration=duration, peak_rss=rss,
                                  cpu_percent=2, ended_early=early)
            self.assertEqual(result["status"], expected)


if __name__ == "__main__":
    unittest.main()
