"""G-03.10: deterministická velikost i vstupní validace 5000 identit."""
import importlib.util
import json
import tempfile
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location(
    "g03_sse_benchmark", ROOT / "tools/benchmark_g03_sse.py"
)
benchmark = importlib.util.module_from_spec(spec)
spec.loader.exec_module(benchmark)


class FiveThousandScenarioTests(unittest.TestCase):
    def test_five_thousand_fixture_is_sparse_and_unique(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "readsb.json"
            benchmark.write_fixture(target, 5000, 1, compact=True)
            doc = json.loads(target.read_text(encoding="utf-8"))
            self.assertEqual(len(doc["aircraft"]), 5000)
            self.assertEqual(len({a["hex"] for a in doc["aircraft"]}), 5000)
            self.assertEqual(doc["aircraft"][0]["hex"], "000001")
            self.assertEqual(doc["aircraft"][-1]["hex"], "001388")
            self.assertTrue(all("lat" not in a and "seen" in a for a in doc["aircraft"]))
            self.assertLess(target.stat().st_size, 4 * 1024 * 1024)
            self.assertFalse(target.with_suffix(".tmp").exists())

    def test_scenarios_keep_full_density_and_max_client_gate(self):
        self.assertIn((100, 250), benchmark.SCENARIOS)
        self.assertIn((10, 1000), benchmark.SCENARIOS)
        self.assertIn((100, 5000), benchmark.SCENARIOS)
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "readsb.json"
            benchmark.write_fixture(target, 250, 2)
            row = json.loads(target.read_text(encoding="utf-8"))["aircraft"][0]
            self.assertIn("lat", row)
            self.assertIn("seen_pos", row)
            self.assertIn("alt_baro", row)


if __name__ == "__main__":
    unittest.main()
