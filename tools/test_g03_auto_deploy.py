"""Regrese G-03: main publikace, systemd stage, bounded rollback."""
from pathlib import Path
import unittest
R = Path(__file__).resolve().parent.parent
W = (R / ".github/workflows/g02-dev-candidate.yml").read_text()
I = (R / "deploy/install-dev-bundle.sh").read_text()
B = (R / "deploy/activate-g03-main-autodeploy.sh").read_text()

class G03AutomaticDeployment(unittest.TestCase):
    def test_main_auto_publishes_and_deploys(self):
        self.assertNotIn("g03_runtime == 'false'", W)
        self.assertIn("needs: publish-dev-candidate", W)
        self.assertIn("G04_STAGE_GATE_V3", W)
    def test_root_install_is_one_time_and_safe(self):
        self.assertIn("refs/remotes/origin/main", B)
        self.assertIn("G04_STAGE_GATE_V3", B)
        self.assertIn("airradarv2-main-deploy-install.sh", B)
        self.assertNotIn("systemctl restart", B)
    def test_mode_and_release_roll_back_together(self):
        for marker in ("mode_changed=0", "mode_was_present=0", "mode_backup",
                       "systemctl daemon-reload || true",
                       'ln -sfn "$old" "$base/.rollback-next"',
                       "ExecStart=/opt/airradar-v2/current/airradarv2-engine --g03-live-api",
                       "AIRRADAR_V2_BEAST_ADDR=$receiver:30005",
                       "verify-airradarv2-runtime.py"):
            self.assertIn(marker, I)
        self.assertIn("--g04-live-history", I)
        self.assertIn("runtime-stage.txt", I)
    def test_does_not_mutate_v1(self):
        self.assertNotIn("Boym323/AirRadar/", B)
        self.assertNotIn("systemctl restart readsb", I)
if __name__ == "__main__":
    unittest.main()
