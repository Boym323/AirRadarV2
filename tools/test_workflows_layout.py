"""Regresní ochrana minimalistické CI/CD topologie AirRadarV2.

Tři aktivní workflow: CI pro PR, DEV release/deploy přes runner, denní statistiky.
Historická G-02 diagnostika a ruční duplicate-package zůstávají v Git historii.
"""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent.parent
WF = ROOT / ".github" / "workflows"


class WorkflowTopology(unittest.TestCase):
    def test_three_workflows_and_no_historical_duplicates(self):
        actual = {f.name for f in WF.glob("*.yml")}
        self.assertEqual(actual, {
            "ci.yml",
            "g02-dev-candidate.yml",
            "development-metrics.yml",
        })

    def test_single_canonical_validation_suite(self):
        ci = (WF / "ci.yml").read_text(encoding="utf-8")
        release = (WF / "g02-dev-candidate.yml").read_text(encoding="utf-8")
        self.assertIn("  pull_request:", ci)
        self.assertIn("  workflow_call:", ci)
        self.assertNotIn("  push:", ci)
        for required in (
            "cargo clippy --locked", "cargo test --locked",
            "cargo run --locked --quiet -p airradarv2-engine -- --g02-replay",
            "cargo run --locked --quiet -p airradarv2-engine -- --g02-benchmark",
            "npm run lint", "npm run typecheck", "npm test", "npm run build",
            "node tools/check-docs.mjs", "tools/validate-g01-schemas.py",
            "test_main_dev_auto.py", "test_g02_live_auto.py",
        ):
            self.assertIn(required, ci)
        self.assertIn("uses: ./.github/workflows/ci.yml", release)
        self.assertIn("needs: validate", release)
        self.assertEqual(release.count("cargo test --locked"), 0)
        self.assertEqual(release.count("cargo clippy --locked"), 0)
        self.assertEqual(release.count("npm test"), 0)
        self.assertIn("cargo build --locked --release", release)
        self.assertIn("actions/upload-artifact@v4", release)
        self.assertIn("  publish-dev-candidate:", release)
        self.assertIn("  deploy-dev:", release)
        self.assertNotIn("  verify-dev-lxc:", release)
        self.assertIn("runs-on: [self-hosted, linux, x64, airradarv2-dev]", release)
        self.assertIn("sudo -n /usr/local/libexec/airradarv2-runner-deploy.sh", release)
        self.assertIn("Ověřit právě nasazený release a lokální služby", release)
        self.assertIn("github.ref == 'refs/heads/main'", release)

    def test_g03_main_commit_never_publishes_or_deploys(self):
        release = (WF / "g02-dev-candidate.yml").read_text(encoding="utf-8")
        self.assertIn("g03_runtime: ${{ steps.stage.outputs.g03_runtime }}", release)
        self.assertIn("id: stage", release)
        self.assertIn("if test -f contracts/openapi.g03.yaml; then", release)
        self.assertIn("grep -qx 'G-03' package/web/g03-runtime-required.txt", release)
        self.assertIn("echo 'g03_runtime=true' >> \"$GITHUB_OUTPUT\"", release)
        self.assertIn("if: needs.test-and-build.outputs.g03_runtime == 'false'", release)
        self.assertIn("  deploy-dev:\n", release)
        self.assertIn("needs: publish-dev-candidate", release)

    def test_daily_metrics_does_not_restart_dev(self):
        metrics = (WF / "development-metrics.yml").read_text(encoding="utf-8")
        release = (WF / "g02-dev-candidate.yml").read_text(encoding="utf-8")
        self.assertIn("  schedule:", metrics)
        self.assertIn("  workflow_dispatch:", metrics)
        self.assertNotIn("  push:", metrics)
        self.assertIn("    paths-ignore:", release)
        self.assertIn("      - 'README.md'", release)
        self.assertIn("      - 'docs/metrics/**'", release)
        self.assertIn("branches: [main]", release)
        self.assertNotIn("codex/g02-auto-lxc-20261022", release)


if __name__ == "__main__":
    unittest.main()
