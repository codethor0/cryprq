"""Regression checks for failure propagation; no Docker daemon or network required."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import re
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
SUCCESS = 'E2E_RESULT passed=2 failed=0 skipped=0'


class ProbeExitTests(unittest.TestCase):
    def run_probe(self, *, fail_build=False, docker_available=False,
                  suite_exit=0, summary=SUCCESS, auth_exit=0, auth_binary=True):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'scripts').mkdir()
            (root / 'web').mkdir()
            (root / 'docker/e2e').mkdir(parents=True)
            (root / 'docker/e2e/test_suite.sh').write_text(
                'printf "%s\\n" "$PROBE_SUITE_SUMMARY"\n' + f'exit {suite_exit}\n')
            shutil.copyfile(ROOT / 'scripts/ci_e2e_probe.sh', root / 'scripts/ci_e2e_probe.sh')
            # Stand-ins for the authenticated E2E scripts and the release binary.
            for name in ['e2e-authenticated-vpn.sh', 'e2e-authenticated-file-transfer.sh']:
                (root / 'scripts' / name).write_text(f'exit {auth_exit}\n')
            if auth_binary:
                (root / 'target/release').mkdir(parents=True)
                binary = root / 'target/release/cryprq'
                binary.write_text('#!/bin/sh\nexit 0\n')
                binary.chmod(0o755)
            bin_dir = root / 'bin'
            bin_dir.mkdir()
            commands = {
                'git': 'exit 0',
                'python3': 'exit 0',
                'cargo': 'exit 0',
                'npm': f'exit {1 if fail_build else 0}',
                'timeout': 'shift; exec "$@"',
                'docker': f'''if [ "$1" = info ]; then exit {0 if docker_available else 1}; fi
if [ "$1" = compose ]; then
  printf '%s\\n' "$*" >> "$PROBE_COMMAND_LOG"
fi
exit 0''',
            }
            for name, body in commands.items():
                script = bin_dir / name
                script.write_text('#!/bin/sh\n' + body + '\n')
                script.chmod(0o755)
            command_log = root / 'commands.log'
            env = {**os.environ, 'PATH': str(bin_dir) + os.pathsep + os.environ['PATH'],
                   'PROBE_COMMAND_LOG': str(command_log), 'PROBE_SUITE_SUMMARY': summary}
            result = subprocess.run(['bash', str(root / 'scripts/ci_e2e_probe.sh')],
                                    cwd=root, env=env, text=True, capture_output=True)
            return result, command_log.read_text() if command_log.exists() else ''

    def test_failed_build_is_not_pass(self):
        result, _ = self.run_probe(fail_build=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn('npm build:         FAIL', result.stdout)

    def test_missing_docker_is_incomplete(self):
        result, _ = self.run_probe()
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn('NOT_RUN_ENV_LIMITATION', result.stdout)

    def test_failed_runner_cannot_be_masked_by_success_text(self):
        result, commands = self.run_probe(docker_available=True, suite_exit=7)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn('e2e run:           FAIL', result.stdout)
        self.assertIn('up --build -d listener dialer', commands)

    def test_exact_success_summary_is_required(self):
        for summary in ['', 'Passed: 50\nFailed: 01',
                        'E2E_RESULT passed=50 failed=01 skipped=0',
                        'E2E_RESULT passed=5 failed=0 skipped=0',
                        'E2E_RESULT passed=2 failed=0 skipped=1']:
            with self.subTest(summary=summary):
                result, _ = self.run_probe(docker_available=True, summary=summary)
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)

    def test_valid_success_propagates(self):
        result, _ = self.run_probe(docker_available=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_absent_test_directory_cannot_pass(self):
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(['bash', str(ROOT / 'docker/e2e/test_suite.sh')],
                                    text=True, capture_output=True,
                                    env={**os.environ, 'TESTS_DIR': directory})
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('Required test is missing', result.stdout)
            self.assertNotIn('All tests passed', result.stdout)

    def test_failed_authenticated_e2e_is_not_pass(self):
        result, _ = self.run_probe(docker_available=True, auth_exit=3)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn('vpn link:          FAIL', result.stdout)
        self.assertIn('file transfer:     FAIL', result.stdout)

    def test_missing_binary_fails_authenticated_e2e(self):
        result, _ = self.run_probe(docker_available=True, auth_binary=False)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn('vpn link:          FAIL', result.stdout)

    def test_extra_or_missing_legacy_test_cannot_pass(self):
        # The harness runs exactly its required tests; a directory holding
        # only one of them fails before anything runs.
        with tempfile.TemporaryDirectory() as directory:
            shutil.copyfile(ROOT / 'docker/e2e/tests/01_legacy_quic_connection.sh',
                            Path(directory) / '01_legacy_quic_connection.sh')
            result = subprocess.run(['bash', str(ROOT / 'docker/e2e/test_suite.sh')],
                                    text=True, capture_output=True,
                                    env={**os.environ, 'TESTS_DIR': directory})
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('Required test is missing', result.stdout)


class CoverageMapTests(unittest.TestCase):
    """The retired placeholders stay retired and what replaced them stays in place."""

    @classmethod
    def setUpClass(cls):
        cls.map = json.loads((ROOT / 'docker/e2e/coverage_map.json').read_text())
        cls.workflow = (ROOT / '.github/workflows/e2e-tests.yml').read_text()

    def test_every_replacement_assertion_exists(self):
        self.assertEqual(len(self.map['retired']), 3)
        for retired in self.map['retired']:
            kinds = {entry['kind'] for entry in retired['replaced_by']}
            # Each claim is covered both by a protocol test and end to end.
            self.assertEqual(kinds, {'rust', 'e2e'}, retired['legacy'])
            for entry in retired['replaced_by']:
                with self.subTest(legacy=retired['legacy'], needle=entry['needle']):
                    text = (ROOT / entry['file']).read_text()
                    self.assertIn(entry['needle'], text)

    def test_rust_replacements_are_real_tests(self):
        for retired in self.map['retired']:
            for entry in retired['replaced_by']:
                if entry['kind'] != 'rust':
                    continue
                with self.subTest(needle=entry['needle']):
                    text = (ROOT / entry['file']).read_text()
                    before = text[:text.index(entry['needle'])].rstrip().splitlines()[-1].strip()
                    self.assertIn(before, ['#[test]', '#[tokio::test]'])
                    self.assertNotIn('#[ignore', text[text.index(entry['needle']) - 200:
                                                      text.index(entry['needle'])])

    def test_retired_placeholders_are_gone_and_documented(self):
        tests_dir = ROOT / 'docker/e2e/tests'
        present = sorted(path.name for path in tests_dir.glob('*.sh'))
        self.assertEqual(present, sorted(self.map['legacy_kept']))
        doc = (ROOT / 'docs/E2E_COVERAGE_MAP.md').read_text()
        suite = (ROOT / 'docker/e2e/test_suite.sh').read_text()
        for retired in self.map['retired']:
            self.assertFalse((tests_dir / retired['legacy']).exists())
            self.assertIn(retired['legacy'], doc)
            self.assertNotIn(retired['legacy'], suite)
        for kept in self.map['legacy_kept']:
            self.assertIn(kept, suite)
        self.assertNotIn('UNRESOLVED: no ', ''.join(
            (tests_dir / name).read_text() for name in present))

    def test_legacy_suite_requires_exactly_its_tests(self):
        suite = (ROOT / 'docker/e2e/test_suite.sh').read_text()
        self.assertIn(f"REQUIRED_COUNT={len(self.map['legacy_kept'])}", suite)
        self.assertIn('[ "$PASSED" -eq "$REQUIRED_COUNT" ]', suite)
        self.assertIn('[ "$SKIPPED" -eq 0 ]', suite)
        probe = (ROOT / 'scripts/ci_e2e_probe.sh').read_text()
        self.assertIn(f"E2E_RESULT passed={len(self.map['legacy_kept'])} failed=0 skipped=0", probe)

    def job(self, name):
        match = re.search(rf'^  {re.escape(name)}:\n(.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)',
                          self.workflow, re.S | re.M)
        self.assertIsNotNone(match, f'job {name} is missing')
        return match.group(1)

    def test_workflow_runs_the_authenticated_scripts_unconditionally(self):
        job = self.job('vpn-e2e')
        for script in self.map['required_e2e_scripts']:
            with self.subTest(script=script):
                self.assertTrue((ROOT / script).is_file())
                self.assertIn(f'run: bash {script} ./target/release/cryprq', job)
        self.assertIn('cargo build --release -p cryprq --locked', job)
        # Nothing that turns a failure into a pass, and no condition on the job
        # or on the steps that run the scripts.
        self.assertNotIn('continue-on-error', job)
        self.assertNotIn('|| true', job)
        self.assertNotIn('CRYPRQ_TEST_MODE', job)
        conditions = re.findall(r'^\s+if: (.*)$', job, re.M)
        self.assertEqual(conditions, ['always()'])  # only the log upload
        self.assertNotIn('paths', self.workflow.split('jobs:')[0])

    def test_no_workflow_uses_the_runner_context_outside_steps(self):
        # GitHub rejects the whole workflow file ("workflow file issue", no
        # job is created) when `runner.*` appears in a workflow-level or
        # job-level key such as `env`. It is only defined inside steps.
        for path in sorted((ROOT / '.github/workflows').glob('*.yml')):
            steps_indent = None      # indentation of the enclosing `steps:` key
            for number, line in enumerate(path.read_text().splitlines(), 1):
                stripped = line.strip()
                if not stripped or stripped.startswith('#'):
                    continue
                indent = len(line) - len(line.lstrip())
                # A step list may sit at the same indentation as its key.
                if steps_indent is not None and (
                        indent < steps_indent
                        or (indent == steps_indent and not stripped.startswith('- '))):
                    steps_indent = None          # left that job's steps
                if stripped == 'steps:':
                    steps_indent = indent
                    continue
                if 'runner.' in line and '${{' in line and steps_indent is None:
                    self.fail(f'{path.name}:{number}: runner context outside steps: {stripped}')

    def test_workflow_keeps_the_device_and_legacy_jobs(self):
        self.assertIn('bash scripts/e2e-tun-docker.sh', self.job('vpn-tun-containers'))
        self.assertNotIn('continue-on-error', self.job('vpn-tun-containers'))
        self.assertIn('bash docker/e2e/test_suite.sh', self.job('e2e-tests'))


if __name__ == '__main__':
    unittest.main()
