"""Lockfile MSRV policy: transitive packages may not exceed the pinned toolchain."""

from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import check_lockfile_msrv as msrv  # noqa: E402


def metadata(*packages):
    return {"packages": [dict(name=n, version=v, rust_version=r) for n, v, r in packages]}


class LockfileMsrvTests(unittest.TestCase):
    def test_package_above_msrv_is_a_violation(self):
        found, _ = msrv.violations(metadata(("quinn-proto", "0.11.15", "1.85")), "1.83.0")
        self.assertEqual(found, ["quinn-proto 0.11.15 requires Rust 1.85"])

    def test_packages_at_or_below_msrv_pass(self):
        found, _ = msrv.violations(
            metadata(("a", "1.0.0", "1.83"), ("b", "1.0.0", "1.83.0"), ("c", "1.0.0", "1.74.1")),
            "1.83.0",
        )
        self.assertEqual(found, [])

    def test_patch_level_comparison(self):
        found, _ = msrv.violations(metadata(("a", "1.0.0", "1.83.1")), "1.83.0")
        self.assertEqual(len(found), 1)

    def test_undeclared_rust_version_is_reported_not_failed(self):
        found, undeclared = msrv.violations(metadata(("old", "0.1.0", None)), "1.83.0")
        self.assertEqual((found, undeclared), ([], ["old 0.1.0"]))

    def test_unparseable_versions_fail_closed(self):
        with self.assertRaises(ValueError):
            msrv.violations(metadata(("a", "1.0.0", "1.85-nightly")), "1.83.0")
        with self.assertRaises(ValueError):
            msrv.parse_version("stable")

    def test_msrv_is_read_from_toolchain_file(self):
        with tempfile.TemporaryDirectory() as root:
            Path(root, "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.83.0"\n')
            self.assertEqual(msrv.toolchain_msrv(root), "1.83.0")
            Path(root, "rust-toolchain.toml").write_text("[toolchain]\n")
            with self.assertRaises(ValueError):
                msrv.toolchain_msrv(root)


if __name__ == "__main__":
    unittest.main()
