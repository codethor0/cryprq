#!/usr/bin/env python3
"""
CrypRQ Code Freeze Validation
Ensures repository stability before release.
Validates all critical paths.
"""

import subprocess
import sys
from pathlib import Path

def run_command(cmd, cwd=None):
    """Run a command and return (success, output)."""
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            capture_output=True,
            text=True,
            cwd=cwd
        )
        return result.returncode == 0, result.stdout + result.stderr
    except Exception as e:
        return False, str(e)

def check_git_clean():
    """Check git working tree is clean."""
    print("Checking git working tree...")
    success, output = run_command("git diff --exit-code")
    if not success:
        print("ERROR: Git working tree has uncommitted changes")
        print(output)
        return False
    
    success, output = run_command("git diff --cached --exit-code")
    if not success:
        print("ERROR: Git has staged changes")
        print(output)
        return False
    
    print("PASS: Git working tree is clean")
    return True

def check_build_release():
    """Check release build succeeds."""
    print("Checking release build...")
    success, output = run_command("cargo build --release --workspace")
    if not success:
        print("ERROR: Release build failed")
        print(output)
        return False
    print("PASS: Release build")
    return True

def check_all_tests():
    """Check all tests pass."""
    print("Checking all tests...")
    success, output = run_command("cargo test --workspace --release")
    if not success:
        print("ERROR: Tests failed")
        print(output)
        return False
    print("PASS: All tests")
    return True

def check_cargo_deny():
    """Check cargo deny."""
    print("Checking cargo deny...")
    success, output = run_command("cargo deny check")
    if not success:
        print("ERROR: Cargo deny check failed")
        print(output)
        return False
    print("PASS: Cargo deny")
    return True

def check_formatting():
    """Check formatting."""
    print("Checking formatting...")
    success, output = run_command("cargo fmt --all --check")
    if not success:
        print("ERROR: Formatting check failed")
        print(output)
        return False
    print("PASS: Formatting")
    return True

def check_clippy():
    """Check clippy."""
    print("Checking clippy...")
    success, output = run_command("cargo clippy --all-targets --all-features -- -D warnings")
    if not success:
        print("ERROR: Clippy check failed")
        print(output)
        return False
    print("PASS: Clippy")
    return True

def main():
    """Run all code freeze validations."""
    print("=" * 60)
    print("CrypRQ Code Freeze Validation")
    print("=" * 60)
    print()
    
    checks = [
        ("Git Clean", check_git_clean),
        ("Release Build", check_build_release),
        ("All Tests", check_all_tests),
        ("Cargo Deny", check_cargo_deny),
        ("Formatting", check_formatting),
        ("Clippy", check_clippy),
    ]
    
    failed_checks = []
    for name, check_func in checks:
        if not check_func():
            failed_checks.append(name)
        print()
    
    if failed_checks:
        print("=" * 60)
        print("CODE FREEZE VALIDATION: FAILED")
        print("=" * 60)
        print(f"Failed checks: {', '.join(failed_checks)}")
        print()
        print("Repository is not ready for release.")
        sys.exit(1)
    
    print("=" * 60)
    print("CODE FREEZE VALIDATION: PASSED")
    print("=" * 60)
    print("Repository is ready for release.")
    sys.exit(0)

if __name__ == "__main__":
    main()

