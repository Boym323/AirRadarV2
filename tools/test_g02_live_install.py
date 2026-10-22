"""Static tests for isolated V2 live acceptance installer."""
from pathlib import Path
import unittest
ROOT = Path(__file__).resolve().parent.parent


class TestInstaller(unittest.TestCase):
    def test_systemd_sandbox_and_fixed_source(self):
        s = (ROOT / "deploy/install-g02-live-auto.sh").read_text()
        for needed in [
            "AIRRADAR_V2_G02_PORT=30005", "User=airradarv2",
            "NoNewPrivileges=yes", "ProtectSystem=strict", "ProtectHome=yes",
            "MemoryMax=384M", "CPUQuota=50%", "RuntimeMaxSec=3900",
            "OnUnitInactiveSec=3min", "G02_SOURCE_REF", "python3 -m py_compile",
        ]:
            self.assertIn(needed, s)
        self.assertNotIn("192.168.1.50", s)


if __name__ == "__main__":
    unittest.main()
