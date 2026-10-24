"""Regrese: bezpečná deduplikace PR CI oproti následnému main."""
import importlib.util
from pathlib import Path
from unittest import TestCase, main, mock
import tempfile

spec = importlib.util.spec_from_file_location('gate', Path(__file__).with_name('ci_reuse_gate.py'))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)

REPO = 'Boym323/AirRadarV2'
SHA = '1' * 40
HEAD = '2' * 40
TREE = '3' * 40


def fake_api(path):
    if path == f'repos/{REPO}/commits/{SHA}':
        return {'commit': {'tree': {'sha': TREE}}}
    if path.startswith(f'repos/{REPO}/commits/{SHA}/pulls'):
        return [{'merged_at': '2026-10-24T16:32:05Z', 'number': 44, 'merge_commit_sha': SHA,
                 'base': {'ref': 'main'}, 'head': {'sha': HEAD, 'repo': {'full_name': REPO}}}]
    if '/actions/workflows/' in path:
        return {'workflow_runs': [{'event': 'pull_request', 'head_sha': HEAD,
                 'status': 'completed', 'conclusion': 'success', 'id': 12,
                 'run_attempt': 1, 'head_repository': {'full_name': REPO}}]}
    if '/actions/runs/12/artifacts' in path:
        return {'artifacts': [{'name': 'validated-ci-12-1', 'id': 99, 'expired': False}]}
    raise AssertionError(path)


class GateTests(TestCase):
    @mock.patch.object(gate, 'gh_json', side_effect=fake_api)
    @mock.patch.object(gate, 'read_proof_archive')
    def test_exact_same_tree_and_successful_pr_ci_can_be_reused(self, read, _):
        read.return_value = dict(workflow='ci.yml', pr=44, head_sha=HEAD,
                                 tree_sha=TREE, run_id=12, run_attempt=1)
        ok, info = gate.can_reuse(REPO, SHA, 'ci.yml')
        self.assertTrue(ok)
        self.assertIn('PR #44', info)

    @mock.patch.object(gate, 'gh_json', side_effect=fake_api)
    @mock.patch.object(gate, 'read_proof_archive')
    def test_different_merge_tree_falls_back_to_full_ci(self, read, _):
        read.return_value = dict(workflow='ci.yml', pr=44, head_sha=HEAD,
                                 tree_sha='4' * 40, run_id=12, run_attempt=1)
        self.assertFalse(gate.can_reuse(REPO, SHA, 'ci.yml')[0])

    @mock.patch.object(gate, 'gh_json', side_effect=fake_api)
    @mock.patch.object(gate, 'read_proof_archive')
    def test_old_run_attempt_not_accepted(self, read, _):
        read.return_value = dict(workflow='ci.yml', pr=44, head_sha=HEAD,
                                 tree_sha=TREE, run_id=12, run_attempt=2)
        self.assertFalse(gate.can_reuse(REPO, SHA, 'ci.yml')[0])

    @mock.patch.object(gate, 'gh_json')
    def test_direct_main_push_has_no_verified_pr(self, api):
        api.side_effect = lambda path: {'commit': {'tree': {'sha': TREE}}} if path.endswith('/commits/' + SHA) else []
        self.assertFalse(gate.can_reuse(REPO, SHA, 'ci.yml')[0])

    @mock.patch.object(gate, 'gh_json', side_effect=fake_api)
    @mock.patch.object(gate, 'read_proof_archive')
    def test_database_checks_require_separate_g04_proof(self, read, _):
        read.return_value = dict(workflow='ci.yml', pr=44, head_sha=HEAD,
                                 tree_sha=TREE, run_id=12, run_attempt=1)
        self.assertFalse(gate.can_reuse(REPO, SHA, 'g04-postgres-migrations.yml')[0])

    def test_output_is_false_by_default(self):
        with tempfile.TemporaryDirectory() as d:
            path = str(Path(d) / 'output')
            gate.output_gate(path, False)
            self.assertEqual(Path(path).read_text(), 'reusable=false\n')

    def test_invalid_revision_is_rejected(self):
        with self.assertRaises(ValueError):
            gate.can_reuse(REPO, 'main', 'ci.yml')


if __name__ == '__main__':
    main()
