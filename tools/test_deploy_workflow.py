"""Regresní kontrola, že běžný deploy není závislý na G-02 veřejném reportu."""

from pathlib import Path
import unittest


WORKFLOW = Path(__file__).resolve().parents[1] / ".github/workflows/g02-dev-candidate.yml"
TEXT = WORKFLOW.read_text(encoding="utf-8")


class DeployWorkflowTests(unittest.TestCase):
    def test_runner_publishes_actual_activated_sha(self):
        runner = (Path(__file__).resolve().parents[1] / "deploy/runner-deploy.sh").read_text(
            encoding="utf-8"
        )
        self.assertIn("/opt/airradar-v2/current/VERSION", runner)
        self.assertIn("/var/cache/airradarv2-main-deploy/status.json", runner)
        self.assertIn("temporary.replace(public)", runner)
        self.assertIn("sha != expected", runner)

    def test_has_local_post_deploy_gate(self):
        self.assertIn("DEV · Kontrola nasazení", TEXT)
        self.assertIn("/opt/airradar-v2/current", TEXT)
        self.assertIn("airradarv2-engine.service", TEXT)
        self.assertIn("nginx.service", TEXT)
        self.assertIn("127.0.0.1:4101/api/v1/health", TEXT)
        self.assertIn("192.168.1.126:8080/", TEXT)

    def test_no_periodic_g02_public_gate(self):
        self.assertNotIn("verify-dev-lxc:", TEXT)
        self.assertNotIn("g02-status.json", TEXT)

    def test_stage_release_auto_enabled(self):
        self.assertNotIn("g03_runtime == 'false'", TEXT)
        self.assertIn("name: G04 · Kontrola stage", TEXT)
        self.assertIn("deploy/runtime-stage", TEXT)
        self.assertIn("package/web/runtime-stage.txt", TEXT)
        self.assertIn("G04_STAGE_GATE_V3", TEXT)
        self.assertIn("verify-airradarv2-runtime.py", TEXT)

    def test_metrics_serialize_and_cannot_invalidate_valid_app_release(self):
        metrics = WORKFLOW.with_name("development-metrics.yml").read_text(encoding="utf-8")
        self.assertIn("group: airradarv2-dev-release-refs/heads/main", metrics)
        self.assertIn("group: airradarv2-dev-release-${{ github.ref }}", TEXT)
        self.assertIn("cancel-in-progress: false", metrics)
        self.assertIn("compare/${GITHUB_SHA}...${current_main}", TEXT)
        self.assertIn("(.commits | length) == .ahead_by", TEXT)
        self.assertIn('"docs: automatická aktualizace statistik vývoje"', TEXT)
        self.assertIn('.author.login == "github-actions[bot]"', TEXT)
        self.assertIn('.parents[0].sha == $parent', TEXT)
        self.assertIn('(.filename == "README.md" or (.filename | startswith("docs/metrics/")))', TEXT)
        self.assertIn('(.files | length) > 0', TEXT)

    def test_release_and_deploy_dependencies_remain(self):
        self.assertIn("needs: test-and-build", TEXT)
        self.assertIn("needs: publish-dev-candidate", TEXT)
        self.assertIn("always() && needs.test-and-build.result == 'success'", TEXT)
        self.assertIn("always() && needs.publish-dev-candidate.result == 'success'", TEXT)
        self.assertIn("airradarv2-runner-deploy.sh", TEXT)
        self.assertIn("sha256sum -c", TEXT)
        self.assertIn("tar -tzf", TEXT)
        # G-02 syntetické kontroly jsou ve sdíleném ci.yml, ne v release YAML.
        self.assertIn("test_verify_g02_public_status.py", WORKFLOW.with_name("ci.yml").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
