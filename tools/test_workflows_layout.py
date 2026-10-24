"""Regresní ochrana minimalistické CI/CD topologie AirRadarV2.

Čtyři aktivní workflow: CI, DEV release/deploy, denní statistiky, izolovaný G-04 PostgreSQL.
Historická G-02 diagnostika a ruční duplicate-package zůstávají v Git historii.
"""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent.parent
WF = ROOT / ".github" / "workflows"


class WorkflowTopology(unittest.TestCase):
    def test_short_display_names_keep_ci_contracts(self):
        """Názvy v GitHub UI jsou stručné, technická identita zůstává stejná."""
        display_names = []
        for path in sorted(WF.glob("*.yml")):
            for line in path.read_text(encoding="utf-8").splitlines():
                if (line.startswith("name: ") or line.startswith("    name: ")
                        or line.startswith("      - name: ")):
                    label = line.split("name: ", 1)[1]
                    self.assertLessEqual(len(label), 35, f"{path.name}: {label!r}")
                    display_names.append(label)
        self.assertGreaterEqual(len(display_names), 45)
        ci = (WF / "ci.yml").read_text(encoding="utf-8")
        g04 = (WF / "g04-postgres-migrations.yml").read_text(encoding="utf-8")
        dev = (WF / "g02-dev-candidate.yml").read_text(encoding="utf-8")
        self.assertIn("    name: Česká dokumentace", ci)
        self.assertIn("  rust-engine:", ci)
        self.assertIn("  vite-web:", ci)
        self.assertIn("name: validated-ci-${{ github.run_id }}-${{ github.run_attempt }}", ci)
        self.assertIn("validated-g04-postgres-migrations-${{ github.run_id }}-${{ github.run_attempt }}", g04)
        self.assertIn("name: g05-chromium-benchmark-${{ github.sha }}", ci)
        self.assertIn("name: g02-dev-${{ github.sha }}", dev)

    def test_four_workflows_and_no_historical_duplicates(self):
        actual = {f.name for f in WF.glob("*.yml")}
        self.assertEqual(actual, {
            "ci.yml",
            "g02-dev-candidate.yml",
            "development-metrics.yml",
            "g04-postgres-migrations.yml",
        })

    def test_g04_isolated_postgres_migration_never_deploys(self):
        g04 = (WF / "g04-postgres-migrations.yml").read_text(encoding="utf-8")
        self.assertIn("image: postgres:18", g04)
        self.assertIn("POSTGRES_DB: airradar_v2_dev", g04)
        self.assertIn("PGHOST: 127.0.0.1", g04)
        self.assertIn("tools/test_g04_migrations.sh", g04)
        self.assertIn("branches: [main]", g04)
        self.assertNotIn("self-hosted", g04)
        self.assertNotIn("gh release create", g04)
        self.assertNotIn("runner-deploy", g04)
        self.assertIn("needs: pr_reuse", g04)
        self.assertIn("--workflow g04-postgres-migrations.yml", g04)
        self.assertIn("validated-g04-postgres-migrations-", g04)
        self.assertIn("if: always() && github.event_name == 'pull_request' && needs.isolated-postgres.result == 'success'", g04)

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
        self.assertIn("needs: [pr_reuse, validate]", release)
        self.assertIn("needs.pr_reuse.outputs.reusable == 'true'", release)
        self.assertIn("if: needs.pr_reuse.outputs.reusable != 'true'", release)
        self.assertIn("tools/ci_reuse_gate.py verify", release)
        self.assertIn("tools/ci_reuse_gate.py create", ci)
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
        self.assertIn("DEV · Kontrola nasazení", release)
        self.assertIn("github.ref == 'refs/heads/main'", release)

    def test_g03_main_commit_publishes_and_runs_runner(self):
        release = (WF / "g02-dev-candidate.yml").read_text(encoding="utf-8")
        self.assertNotIn("g03_runtime == 'false'", release)
        self.assertIn("name: G04 · Kontrola stage", release)
        self.assertIn("package/web/runtime-stage.txt", release)
        self.assertIn("deploy/runtime-stage", release)
        self.assertIn("  publish-dev-candidate:", release)
        self.assertIn("needs: publish-dev-candidate", release)
        self.assertIn("G04_STAGE_GATE_V3", release)

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
