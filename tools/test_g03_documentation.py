"""Regrese české dokumentace při uzavření G-03: současný stav versus historie."""
from pathlib import Path
import unittest
ROOT = Path(__file__).resolve().parent.parent
SHA = "29ee2ee75feab9279123b77d50895d648c5419d7"
RUN = "39906245759"

def read(path):
    return (ROOT / path).read_text(encoding="utf-8")

class G03FinalDocumentation(unittest.TestCase):
    def test_current_status_at_document_front(self):
        for path in ("README.md", "docs/reports/G-03-VYSLEDEK.md",
                     "docs/ukoly/G-03-API-A-SSE.md", "docs/runbooks/DEV-MAIN-AUTODEPLOY.md"):
            with self.subTest(path=path):
                front = "\n".join(read(path).splitlines()[:18])
                self.assertTrue("23. 10. 2026" in front or "24. 10. 2026" in front,
                                f"{path}: chybí datovaný aktuální nebo přejímací stav")
                self.assertIn("G-03", front)
                self.assertTrue("HOTOVO" in front or "ACCEPTED" in front)
                self.assertTrue(
                    RUN in front or "G-04/J09" in front,
                    f"{path} neobsahuje historický G-03 důkaz ani aktuální G-04/J09 stav",
                )

    def test_stages_and_keep_evidence(self):
        stages = read("docs/plan/ETAPY-A-STAV.md")
        g03 = next(line for line in stages.splitlines() if line.startswith("| G-03 |"))
        g13 = next(line for line in stages.splitlines() if line.startswith("| G-13 |"))
        for value in ("HOTOVO", "DEV ACCEPTED", RUN):
            self.assertIn(value, g03)
        self.assertIn("PRODUKČNÍ ETAPA NEZAHÁJENA", g13)
        self.assertIn("DEV auto-deploy je již aktivní", g13)
        matrix = read("docs/plan/MATICE-FUNKCI.md")
        for item in ("J06", "J07", "J08"):
            row = next(v for v in matrix.splitlines() if chr(96) + item + chr(96) in v)
            self.assertIn("TESTED", row)

    def test_archive_is_marked_historical(self):
        for path in ("docs/audits/G-03-PRED-MERGE-2026-10-22.md",
                     "docs/reports/DEV-MAIN-AUTODEPLOY-2026-10-22.md"):
            with self.subTest(path=path):
                self.assertIn("historick", read(path)[:800].lower())
                self.assertIn(RUN, read(path)[:1000])
        self.assertIn("g04-postgres-migrations.yml",
                      read("docs/runbooks/DEV-MAIN-AUTODEPLOY.md"))

    def test_dev_is_not_production(self):
        report = read("docs/reports/G-03-VYSLEDEK.md")
        self.assertIn(SHA, report[:3000])
        self.assertIn("úmyslný", report[:3000].lower())
        self.assertIn("G-13", report[:3000])
        self.assertIn("G-04/J09", read("README.md"))
        self.assertIn("G-13", read("README.md"))

if __name__ == "__main__":
    unittest.main()
