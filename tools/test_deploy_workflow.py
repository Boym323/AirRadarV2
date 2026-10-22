"""Regresní kontrola, že běžný deploy není závislý na G-02 veřejném reportu."""

from pathlib import Path
import unittest


WORKFLOW = Path(__file__).resolve().parents[1] / ".github/workflows/g02-dev-candidate.yml"
TEXT = WORKFLOW.read_text(encoding="utf-8")


class DeployWorkflowTests(unittest.TestCase):
    def test_has_local_post_deploy_gate(self):
        self.assertIn("Ověřit právě nasazený release a lokální služby", TEXT)
        self.assertIn("/opt/airradar-v2/current", TEXT)
        self.assertIn("airradarv2-engine.service", TEXT)
        self.assertIn("nginx.service", TEXT)
        self.assertIn("127.0.0.1:4101/api/v1/health", TEXT)
        self.assertIn("192.168.1.126:8080/", TEXT)

    def test_no_periodic_g02_public_gate(self):
        self.assertNotIn("verify-dev-lxc:", TEXT)
        self.assertNotIn("g02-status.json", TEXT)

    def test_g03_release_is_fail_closed(self):
        self.assertIn("g03_runtime: ${{ steps.stage.outputs.g03_runtime }}", TEXT)
        self.assertIn("if: ${{ needs['test-and-build'].outputs.g03_runtime == 'false' }}", TEXT)
        self.assertIn("test -f package/web/g03-runtime-required.txt", TEXT)
        self.assertIn("grep -qx 'G-03' package/web/g03-runtime-required.txt", TEXT)
        self.assertIn("echo 'g03_runtime=true'", TEXT)

    def test_release_and_deploy_dependencies_remain(self):
        self.assertIn("needs: test-and-build", TEXT)
        self.assertIn("needs: publish-dev-candidate", TEXT)
        self.assertIn("airradarv2-runner-deploy.sh", TEXT)
        self.assertIn("sha256sum -c", TEXT)
        self.assertIn("tar -tzf", TEXT)
        self.assertIn("test_verify_g02_public_status.py", TEXT)


if __name__ == "__main__":
    unittest.main()
