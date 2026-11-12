#!/usr/bin/env python3
"""
CrypRQ Stability Monitor
Monitors AST stability, formatting drift, dependency graph changes,
build stability, test stability, and architecture consistency.
"""

import subprocess
import sys
import json
from datetime import datetime

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

def check_formatting_stability():
    """Report formatting drift without changing the monitored source."""
    print("Checking formatting stability...")
    # A monitor must report drift without silently formatting it away.
    success, output = run_command("cargo fmt --all --check")
    if not success:
        print("ERROR: Formatting drift detected or formatting check failed")
        print(output)
        return False
    print("PASS: Formatting is stable")
    return True

def check_build_stability():
    """Check build stability."""
    print("Checking build stability...")
    success, output = run_command("cargo build --workspace")
    if not success:
        print("ERROR: Build failed")
        print(output)
        return False
    print("PASS: Build is stable")
    return True

def check_test_stability():
    """Check test stability."""
    print("Checking test stability...")
    success, output = run_command("cargo test --workspace")
    if not success:
        print("ERROR: Tests failed")
        print(output)
        return False
    print("PASS: Tests are stable")
    return True

def check_dependency_graph():
    """Check dependency graph consistency."""
    print("Checking dependency graph...")
    success, output = run_command("cargo tree --workspace")
    if not success:
        print("ERROR: Dependency graph check failed")
        print(output)
        return False
    
    # Keep warnings nonblocking according to cargo-deny policy, but propagate
    # scanner errors, policy violations, and missing tooling.
    success2, output2 = run_command("cargo deny check")
    print(output2)
    if not success2:
        print("ERROR: Dependency policy check failed")
        return False
    print("PASS: Dependency graph check")
    return True

def check_architecture_consistency():
    """Check architecture consistency (no cycles, proper layering)."""
    print("Checking architecture consistency...")
    
    success, output = run_command("cargo metadata --no-deps --format-version 1 --locked")
    if not success:
        print("ERROR: Could not inspect workspace architecture")
        print(output)
        return False

    allowed_dependencies = {
        "cryprq-crypto": set(),
        "cryp-rq-core": {"cryprq-crypto"},
        "node": {"cryprq-crypto", "cryp-rq-core"},
        "p2p": {"cryprq-crypto", "node"},
        "cryprq": {"cryprq-crypto", "p2p", "node"},
    }
    try:
        metadata = json.loads(output)
        members = set(metadata["workspace_members"])
        packages = [package for package in metadata["packages"] if package["id"] in members]
        names = {package["name"] for package in packages}
        if names != set(allowed_dependencies):
            print("ERROR: Workspace members differ from the documented architecture")
            return False
        for package in packages:
            internal_dependencies = {dependency["name"] for dependency in package["dependencies"]} & names
            unexpected = internal_dependencies - allowed_dependencies[package["name"]]
            if unexpected:
                print(f"ERROR: {package['name']} has forbidden workspace dependencies: {sorted(unexpected)}")
                return False
    except (ValueError, KeyError, TypeError) as error:
        print(f"ERROR: Invalid workspace metadata: {error}")
        return False
    print("PASS: Architecture consistency")
    return True

def generate_report():
    """Generate stability report."""
    report = {
        "timestamp": datetime.now().isoformat(),
        "checks": {
            "formatting_stability": check_formatting_stability(),
            "build_stability": check_build_stability(),
            "test_stability": check_test_stability(),
            "dependency_graph": check_dependency_graph(),
            "architecture_consistency": check_architecture_consistency(),
        }
    }
    
    return report

def main():
    """Run stability monitoring."""
    print("=" * 60)
    print("CrypRQ Stability Monitor")
    print("=" * 60)
    print()
    
    report = generate_report()
    
    all_passed = all(report["checks"].values())
    
    print()
    print("=" * 60)
    print("Stability Report")
    print("=" * 60)
    for check, passed in report["checks"].items():
        status = "PASS" if passed else "FAIL"
        print(f"{check}: {status}")
    
    print()
    if all_passed:
        print("=" * 60)
        print("STABILITY MONITOR: PASSED")
        print("=" * 60)
        sys.exit(0)
    else:
        print("=" * 60)
        print("STABILITY MONITOR: FAILED")
        print("=" * 60)
        sys.exit(1)

if __name__ == "__main__":
    main()

