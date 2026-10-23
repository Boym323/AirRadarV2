import json
import tempfile
import unittest
from pathlib import Path
import importlib.util

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("evaluate", ROOT / "tools/evaluate_g04_canary.py")
evaluate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evaluate)


class CanaryEvaluation(unittest.TestCase):
    def test_rejects_loss_timeout_and_missing_operational_evidence(self):
        snapshots = [{"pending": 0, "inflight": 0, "eligible": 100,
                      "dropped_writer": 2, "sql_timeouts": 1, "connection": "up"}]
        result = evaluate.evaluate(snapshots, {"duration_seconds": 1800})
        self.assertEqual(result["status"], "FAIL")
        self.assertFalse(result["checks"]["candidate_loss_le_1_percent"])

    def test_accepts_complete_fixture(self):
        snapshots = [{"pending": 4, "inflight": 1, "eligible": 1000,
                      "dropped_writer": 0, "sql_timeouts": 0, "connection": "up"},
                     {"pending": 0, "inflight": 0, "eligible": 1000,
                      "dropped_writer": 0, "sql_timeouts": 0, "connection": "up"}]
        metrics = {"duration_seconds": 1800, "actual_db_write": True,
                   "g03_sse_healthy": True, "cpu_ram_recorded": True,
                   "wal_recorded": True}
        self.assertEqual(evaluate.evaluate(snapshots, metrics)["status"], "PASS")


if __name__ == "__main__":
    unittest.main()
