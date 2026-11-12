#!/usr/bin/env python3
"""Regression tests for scheduled checks that must fail on incomplete validation."""

import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import textwrap
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
WORKFLOWS = ROOT / '.github' / 'workflows'
spec = importlib.util.spec_from_file_location('stability_monitor', ROOT / 'scripts' / 'stability_monitor.py')
monitor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(monitor)


def workflow_step(filename, name, key='run'):
    """Extract a literal script from the actual workflow, without YAML dependencies."""
    text = (WORKFLOWS / filename).read_text()
    blocks = re.split(r'^    - name: ', text, flags=re.MULTILINE)
    block = next(block for block in blocks if block.splitlines()[0] == name)
    lines = block.splitlines()
    start = next(i for i, line in enumerate(lines) if line.strip() == key + ': |')
    indent = len(lines[start]) - len(lines[start].lstrip())
    script = []
    for line in lines[start + 1:]:
        if line.strip() and len(line) - len(line.lstrip()) <= indent:
            break
        script.append(line)
    return textwrap.dedent('\n'.join(script))


def run_shell(script, directory, **env):
    return subprocess.run(
        ['bash', '-e', '-o', 'pipefail', '-c', script],
        cwd=directory, env={**os.environ, **env}, capture_output=True, text=True,
    )


class ScheduledWorkflowTests(unittest.TestCase):
    def quiet_call(self, function, results):
        with patch.object(monitor, 'run_command', side_effect=results) as run, contextlib.redirect_stdout(io.StringIO()):
            result = function()
        return result, run

    def test_dependency_policy_failure_is_blocking(self):
        passed, run = self.quiet_call(monitor.check_dependency_graph,
                                     [(True, 'workspace tree'), (False, 'policy violation')])
        self.assertFalse(passed)
        self.assertEqual(run.call_args_list[-1].args, ('cargo deny check',))

    def test_missing_scanner_is_blocking(self):
        passed, _ = self.quiet_call(monitor.check_dependency_graph,
                                   [(True, 'workspace tree'), (False, '/bin/sh: cargo-deny: command not found')])
        self.assertFalse(passed)

    def test_scanner_warnings_remain_nonblocking_when_scanner_succeeds(self):
        passed, _ = self.quiet_call(monitor.check_dependency_graph,
                                   [(True, 'workspace tree'), (True, 'warning: duplicate versions')])
        self.assertTrue(passed)

    def test_dependency_tree_failure_does_not_pass(self):
        passed, run = self.quiet_call(monitor.check_dependency_graph, [(False, 'resolution failed')])
        self.assertFalse(passed)
        self.assertEqual(run.call_count, 1)

    def test_formatting_drift_is_reported_without_modifying_files(self):
        passed, run = self.quiet_call(monitor.check_formatting_stability, [(False, 'formatting diff')])
        self.assertFalse(passed)
        self.assertEqual(run.call_args_list[0].args, ('cargo fmt --all --check',))
        self.assertEqual(run.call_count, 1)

    def architecture(self, extra_dependency=None):
        names = ['cryprq-crypto', 'cryp-rq-core', 'node', 'p2p', 'cryprq']
        dependencies = {'node': ['cryprq-crypto', 'cryp-rq-core'],
                        'p2p': ['cryprq-crypto', 'node'], 'cryprq': ['p2p', 'node', 'cryprq-crypto']}
        if extra_dependency:
            dependencies['node'].append(extra_dependency)
        return json.dumps({'workspace_members': names,
                           'packages': [{'id': name, 'name': name,
                                         'dependencies': [{'name': dep} for dep in dependencies.get(name, [])]}
                                        for name in names]})

    def test_documented_workspace_layering_passes(self):
        passed, _ = self.quiet_call(monitor.check_architecture_consistency, [(True, self.architecture())])
        self.assertTrue(passed)

    def test_forbidden_dependency_cycle_is_rejected(self):
        passed, _ = self.quiet_call(monitor.check_architecture_consistency, [(True, self.architecture('p2p'))])
        self.assertFalse(passed)

    def test_missing_or_malformed_architecture_metadata_is_rejected(self):
        for success, output in ((False, 'cargo metadata failed'), (True, '{}'), (True, 'not json')):
            with self.subTest(output=output):
                passed, _ = self.quiet_call(monitor.check_architecture_consistency, [(success, output)])
                self.assertFalse(passed)


if __name__ == '__main__':
    unittest.main()
