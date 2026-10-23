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
        snapshots = [self.snapshot(0, eligible=100, committed=98,
                                   dropped_writer=2, sql_timeouts=1)]
        result = evaluate.evaluate(snapshots, {"duration_seconds": 1800})
        self.assertEqual(result["status"], "FAIL")
        self.assertFalse(result["checks"]["candidate_loss_le_1_percent"])

    @staticmethod
    def snapshot(at, pending=0, inflight=0, eligible=1000, committed=1000,
                 dropped_capacity=0, dropped_stale=0, dropped_writer=0,
                 sql_timeouts=0, connection="up"):
        return {"observed_at": at, "pending": pending, "inflight": inflight,
                "eligible": eligible, "committed": committed,
                "dropped_capacity": dropped_capacity, "dropped_stale": dropped_stale,
                "dropped_writer": dropped_writer, "sql_timeouts": sql_timeouts,
                "connection": connection}

    def complete_metrics(self):
        return {"started_at": 1_700_000_000, "finished_at": 1_700_001_800,
                "db_readback_verified": True, "db_readback_rows": 12,
                "g03_sse_healthy": True, "g03_sse_heartbeat_count": 120,
                "cpu_ram_recorded": True, "wal_recorded": True,
                "db_free_space_recorded": True}

    def test_accepts_complete_fixture(self):
        snapshots = [self.snapshot(0, pending=4, inflight=1, committed=995),
                     self.snapshot(600, pending=2, committed=998),
                     self.snapshot(1200, pending=1, committed=999),
                     self.snapshot(1800)]
        metrics = self.complete_metrics()
        self.assertEqual(evaluate.evaluate(snapshots, metrics)["status"], "PASS")

    def test_accepts_initial_unknown_until_first_sql_commit(self):
        snapshots = [
            self.snapshot(0, eligible=0, committed=0, connection="unknown"),
            self.snapshot(600, connection="up"),
            self.snapshot(1200, connection="up"),
            self.snapshot(1800, connection="up"),
        ]
        self.assertEqual(evaluate.evaluate(snapshots, self.complete_metrics())["status"], "PASS")

    def test_rejects_unknown_after_first_sql_commit(self):
        snapshots = [
            self.snapshot(0, eligible=0, committed=0, connection="unknown"),
            self.snapshot(600, connection="up"),
            self.snapshot(1200, connection="unknown"),
            self.snapshot(1800, connection="up"),
        ]
        self.assertFalse(evaluate.evaluate(snapshots, self.complete_metrics())["checks"]["connection_up"])

    def test_rejects_no_sql_commit_during_canary(self):
        snapshots = [
            self.snapshot(i * 600, eligible=0, committed=0, connection="unknown")
            for i in range(4)
        ]
        self.assertFalse(evaluate.evaluate(snapshots, self.complete_metrics())["checks"]["connection_up"])

    def test_rejects_missing_connection(self):
        snapshot = self.snapshot(0)
        del snapshot["connection"]
        self.assertEqual(evaluate.evaluate([snapshot], self.complete_metrics())["status"], "FAIL")

    def test_rejects_persistently_growing_queue(self):
        snapshots = [self.snapshot(i * 600, pending=i * 10, committed=1000 - i * 10)
                     for i in range(4)]
        snapshots[-1]["pending"] = 30
        snapshots[-1]["committed"] = 970
        self.assertFalse(evaluate.evaluate(snapshots, self.complete_metrics())["checks"]["no_persistent_queue_growth"])

    def test_rejects_unavailable_db_and_sql_timeout(self):
        snapshots = [self.snapshot(i * 600, committed=999, connection="down", sql_timeouts=1)
                     for i in range(4)]
        metrics = self.complete_metrics()
        metrics.pop("db_readback_verified")
        result = evaluate.evaluate(snapshots, metrics)
        self.assertFalse(result["checks"]["connection_up"])
        self.assertFalse(result["checks"]["sql_timeouts_zero"])
        self.assertFalse(result["checks"]["actual_db_readback"])

    def test_loss_uses_eligible_as_denominator(self):
        snapshots = [self.snapshot(i * 600, eligible=100, committed=99, dropped_writer=1)
                     for i in range(4)]
        result = evaluate.evaluate(snapshots, self.complete_metrics())
        self.assertEqual(result["observations"]["loss_percent"], 1.0)
        self.assertEqual(result["status"], "PASS")

    def test_rejects_unproven_duration_and_bad_counters(self):
        snapshots = [self.snapshot(i * 10, eligible=100, committed=99, dropped_writer=2)
                     for i in range(4)]
        result = evaluate.evaluate(snapshots, {"duration_seconds": 1800})
        self.assertFalse(result["checks"]["duration_1800_seconds"])
        self.assertFalse(result["checks"]["counter_balance"])


if __name__ == "__main__":
    unittest.main()
