"""Exercise the actual workflow shell against isolated Git histories."""

import os
from pathlib import Path
import subprocess
import tempfile
import textwrap
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/dependency-governance.yml"


def step_script(name):
    lines = WORKFLOW.read_text().splitlines()
    start = lines.index(f"      - name: {name}")
    run = next(i for i in range(start + 1, len(lines)) if lines[i] == "        run: |")
    end = run + 1
    while end < len(lines) and (not lines[end].strip() or lines[end].startswith("          ")):
        end += 1
    return textwrap.dedent("\n".join(lines[run + 1:end])) + "\n"


class DependencyDetectionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.remote = self.root / "remote.git"
        self.repo = self.root / "checkout"
        self.env = dict(os.environ, GIT_AUTHOR_NAME="Fixture", GIT_AUTHOR_EMAIL="fixture@example.invalid",
                        GIT_COMMITTER_NAME="Fixture", GIT_COMMITTER_EMAIL="fixture@example.invalid")
        self.git("init", "--bare", str(self.remote), cwd=self.root)
        self.git("init", str(self.repo), cwd=self.root)
        self.git("remote", "add", "origin", str(self.remote))
        self.base = self.fixture_commit({"README.md": "baseline\n"})
        self.git("push", "origin", f"{self.base}:refs/heads/main")
        self.git("update-ref", "refs/heads/change", self.base)
        self.git("symbolic-ref", "HEAD", "refs/heads/change")
        self.output = self.root / "outputs"

    def git(self, *args, cwd=None, input=None):
        return subprocess.check_output(["git", *args], cwd=cwd or self.repo,
                                       env=self.env, input=input, text=True, stderr=subprocess.PIPE).strip()

    def fixture_commit(self, files, parent=None):
        # Synthetic objects are test fixtures only; no user branch or commit hook is involved.
        entries = []
        for name, content in sorted(files.items()):
            blob = self.git("hash-object", "-w", "--stdin", input=content)
            entries.append(f"100644 blob {blob}\t{name}\n")
        tree = self.git("mktree", input="".join(entries))
        args = ["commit-tree", tree]
        if parent:
            args.extend(["-p", parent])
        return self.git(*args, input="Synthetic governance test fixture\n")

    def detect(self, files=None, unrelated=False, fail_git=None):
        if files is not None:
            head = self.fixture_commit(files, None if unrelated else self.base)
            self.git("update-ref", "refs/heads/change", head)
        env = dict(self.env, GITHUB_BASE_REF="main", GITHUB_OUTPUT=str(self.output))
        if fail_git:
            # Only inject a failure into one Git operation; all history operations remain real.
            import shutil
            executable = shutil.which("git")
            shim = self.root / "bin"
            shim.mkdir(exist_ok=True)
            script = shim / "git"
            script.write_text('#!/bin/bash\nif [ "$1" = "$FAIL_GIT_OPERATION" ]; then exit 73; fi\nexec "$REAL_GIT" "$@"\n')
            script.chmod(0o755)
            env.update(PATH=str(shim) + os.pathsep + env["PATH"],
                       REAL_GIT=executable, FAIL_GIT_OPERATION=fail_git)
        result = subprocess.run(["bash", "-c", step_script("Detect dependency manifest changes")],
                                cwd=self.repo, env=env, text=True, capture_output=True)
        output = self.output.read_text() if self.output.exists() else ""
        return result, output

    def test_manifest_changes_are_detected(self):
        for manifest in ["Cargo.toml", "Cargo.lock", "package.json", "package-lock.json"]:
            with self.subTest(manifest=manifest):
                self.output.unlink(missing_ok=True)
                result, output = self.detect({"README.md": "baseline\n", manifest: "changed\n"})
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("has_manifests=true", output)

    def test_non_manifest_changes_skip_policy(self):
        result, output = self.detect({"README.md": "documentation change\n"})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("has_manifests=false", output)

    def test_unrelated_history_fails_closed(self):
        result, output = self.detect({"Cargo.lock": "changed\n"}, unrelated=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Cannot determine merge base", result.stderr)
        self.assertNotIn("has_manifests=false", output)

    def test_git_failures_never_emit_a_skip(self):
        for operation in ["fetch", "merge-base", "diff"]:
            with self.subTest(operation=operation):
                self.output.unlink(missing_ok=True)
                result, output = self.detect(fail_git=operation)
                self.assertNotEqual(result.returncode, 0)
                self.assertNotIn("has_manifests=false", output)


class MandatoryPolicyInputTests(unittest.TestCase):
    def evaluate(self, missing=(), checker_exit=0, pin='=1.6.0'):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "node").mkdir()
            (root / "governance-artifacts").mkdir()
            (root / "Cargo.toml").write_text('[workspace]\nmembers = ["node"]\n')
            (root / "Cargo.lock").write_text("version = 3\n")
            (root / "node/Cargo.toml").write_text(f'base64ct = "{pin}"\n')
            for filename in missing:
                (root / filename).unlink()
            binary = root / "bin"
            binary.mkdir()
            # Only metadata execution is replaced; the mandatory-input/pin logic
            # comes directly from the workflow and runs under its actual shell.
            checker = binary / "python3"
            checker.write_text(f'#!/bin/sh\necho "metadata checker result"\nexit {checker_exit}\n')
            checker.chmod(0o755)
            output = root / "outputs"
            env = dict(os.environ, GITHUB_OUTPUT=str(output),
                       PATH=str(binary) + os.pathsep + os.environ["PATH"])
            result = subprocess.run(["bash", "-c", step_script("Evaluate MSRV and pin policy")],
                                    cwd=root, env=env, capture_output=True, text=True)
            artifacts = {file.name: file.read_text() for file in (root / "governance-artifacts").iterdir()}
            return result, output.read_text() if output.exists() else "", artifacts

    def test_present_inputs_and_valid_pin_pass(self):
        result, output, _ = self.evaluate()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("msrv_status=OK", output)
        self.assertIn("pin_status=OK", output)

    def test_missing_root_manifests_are_reported_as_msrv_violations(self):
        for missing in [("Cargo.toml",), ("Cargo.lock",), ("Cargo.toml", "Cargo.lock")]:
            with self.subTest(missing=missing):
                result, output, artifacts = self.evaluate(missing=missing)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("msrv_status=VIOLATION", output)
                self.assertIn("pin_status=OK", output)
                self.assertIn("missing", artifacts["lockfile-msrv.txt"])

    def test_missing_node_manifest_is_reported_as_pin_violation(self):
        result, output, artifacts = self.evaluate(missing=("node/Cargo.toml",))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("msrv_status=OK", output)
        self.assertIn("pin_status=VIOLATION", output)
        self.assertIn("missing", artifacts["pin-violations.txt"])

    def test_missing_all_inputs_preserves_both_violation_reports(self):
        result, output, artifacts = self.evaluate(missing=("Cargo.toml", "Cargo.lock", "node/Cargo.toml"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("msrv_status=VIOLATION", output)
        self.assertIn("pin_status=VIOLATION", output)
        self.assertIn("lockfile-msrv.txt", artifacts)
        self.assertIn("pin-violations.txt", artifacts)

    def test_metadata_failure_stays_a_violation_with_artifacts(self):
        result, output, artifacts = self.evaluate(checker_exit=73)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("msrv_status=VIOLATION", output)
        self.assertIn("metadata checker result", artifacts["lockfile-msrv.txt"])

    def test_changed_pin_stays_a_violation_with_artifacts(self):
        result, output, artifacts = self.evaluate(pin='1.6.0')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("pin_status=VIOLATION", output)
        self.assertIn('base64ct = "=1.6.0"', artifacts["pin-violations.txt"])


if __name__ == "__main__":
    unittest.main()
