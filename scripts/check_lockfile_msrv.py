#!/usr/bin/env python3
"""Fail when any locked package declares a rust-version above the pinned toolchain.

The pinned toolchain (rust-toolchain.toml) is the project MSRV. Grepping
Cargo.toml files cannot see transitive lockfile bumps, so this reads the
resolved graph from `cargo metadata --locked`. Packages that declare no
rust-version cannot be checked here and are listed, not failed.
"""

import json
from pathlib import Path
import re
import subprocess
import sys


def parse_version(text):
    parts = text.split(".")
    if not 1 <= len(parts) <= 3 or not all(p.isdigit() for p in parts):
        raise ValueError(f"unparseable Rust version: {text!r}")
    return tuple(int(p) for p in parts) + (0,) * (3 - len(parts))


def toolchain_msrv(root):
    text = (Path(root) / "rust-toolchain.toml").read_text()
    match = re.search(r'^\s*channel\s*=\s*"([^"]+)"', text, re.MULTILINE)
    if not match:
        raise ValueError("rust-toolchain.toml has no channel")
    return match.group(1)


def violations(metadata, msrv):
    limit = parse_version(msrv)
    found, undeclared = [], []
    for package in metadata["packages"]:
        declared = package.get("rust_version")
        name = f'{package["name"]} {package["version"]}'
        if declared is None:
            undeclared.append(name)
        elif parse_version(declared) > limit:
            found.append(f"{name} requires Rust {declared}")
    return sorted(found), sorted(undeclared)


def main(root="."):
    msrv = toolchain_msrv(root)
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--format-version", "1"],
        cwd=root, capture_output=True, text=True,
    )
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        print("ERROR: cargo metadata --locked failed; lockfile MSRV not verified.", file=sys.stderr)
        return 1
    found, undeclared = violations(json.loads(result.stdout), msrv)
    print(f"Lockfile MSRV check against Rust {msrv}: "
          f"{len(found)} violation(s), {len(undeclared)} package(s) without rust-version")
    for line in found:
        print(f"VIOLATION: {line}")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:]))
