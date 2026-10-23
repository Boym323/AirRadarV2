"""Regrese G-03: bootstrap nesmí projít, chyby SSE musí být explicitní."""
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location(
    "g03_verify", ROOT / "deploy/verify-g03-runtime.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

SNAPSHOT = {
    "kind": "snapshot", "epoch": "g03-17-123", "seq": 5,
    "generatedAt": "2026-10-22T20:00:00Z", "aircraft": [],
}


class G03StageGuardTests(unittest.TestCase):
    def test_reject_bootstrap_degraded_and_fake_live(self):
        for status, stage, active in [
            ("bootstrap", "G-00", False),
            ("bootstrap", "G-03", False),
            ("degraded", "G-03", False),
            ("waiting_for_observations", "G-03", True),
            ("live", "G-03", False),
        ]:
            with self.subTest(status=status, stage=stage, active=active):
                with self.assertRaises(ValueError):
                    module.validate_health({
                        "status": status, "stage": stage,
                        "live_ingest": active, "database_connected": False
                    })
        module.validate_health({
            "status": "waiting_for_observations", "stage": "G-03",
            "live_ingest": False, "database_connected": False
        })

    def test_full_initial_snapshot_and_epoch_sequence_are_required(self):
        import json
        raw = (
            f"event: snapshot\nid: g03-17-123:5\ndata: "
            + json.dumps(SNAPSHOT, separators=(",", ":")) + "\n\n"
        ).encode()
        self.assertEqual(module.parse_first_event(raw), SNAPSHOT)
        for invalid in (
            raw.replace(b"event: snapshot", b"event: unavailable"),
            raw.replace(b"id: g03-17-123:5", b"id: other:5"),
            raw.replace(b"data: ", b"junk: "),
            raw.replace(b'"seq":5', b'"seq":-1'),
        ):
            with self.subTest(case=invalid[:50]):
                with self.assertRaises(ValueError):
                    module.parse_first_event(invalid)

    def test_runtime_checks_are_stage_bound_not_bootstrap_allowed(self):
        runtime = (ROOT / "deploy/verify-g03-runtime.py").read_text()
        installer = (ROOT / "deploy/install-dev-bundle.sh").read_text()
        runner = (ROOT / "deploy/install-runner-dev.sh").read_text()
        self.assertIn("expected_stage", runtime)
        self.assertIn("G04_STAGE_GATE_V3", installer)
        self.assertIn("verify-airradarv2-runtime.py", installer)
        self.assertIn("verify-g03-runtime.py", runner)


if __name__ == "__main__":
    unittest.main()
