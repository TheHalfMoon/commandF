#!/usr/bin/env python3
"""Offline counterexamples for the standalone CF11 archive inspector."""
import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

SUT_PATH = Path(__file__).with_name("inspect_archives.py")
spec = importlib.util.spec_from_file_location("cf11_inspect_archives", SUT_PATH)
assert spec and spec.loader
sut = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sut)


def archive(entries):
    """Build isolated synthetic TGZ, no external downloads."""
    out = io.BytesIO()
    with tarfile.open(mode="w:gz", fileobj=out) as tar:
        for name, body, mtime in entries:
            member = tarfile.TarInfo(name)
            member.size = len(body)
            member.mtime = mtime
            tar.addfile(member, io.BytesIO(body))
    return out.getvalue()


MANIFEST = b'{"name":"test.example","version":"1.0.0","dependencies":{"demo":"2.0.0"}}'


class TestArchiveAudit(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="cf11-audit-test-")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.first = self.root / "first.tgz"
        self.second = self.root / "second.tgz"

    def write(self, first, second):
        self.first.write_bytes(first)
        self.second.write_bytes(second)

    def test_identical_archives_have_no_disagreements(self):
        body = archive([("package/package.json", MANIFEST, 0), ("package/resource.json", b"{}", 0)])
        self.write(body, body)
        report = sut.compare(self.first, self.second)
        self.assertTrue(report["raw_archives_identical"])
        self.assertTrue(report["declared_dependencies_equal"])
        self.assertEqual(report["common_content_difference_count"], 0)
        self.assertEqual(report["first_only_regular_files_count"], 0)

    def test_bounded_exact_content_and_metadata_attribution(self):
        a = archive([
            ("package/package.json", MANIFEST, 1),
            ("package/a.json", b"one", 1),
            ("package/unique-a.json", b"x", 1),
        ])
        b = archive([
            ("package/package.json", MANIFEST, 2),
            ("package/a.json", b"two", 2),
            ("package/unique-b.json", b"y", 2),
        ])
        self.write(a, b)
        report = sut.compare(self.first, self.second)
        self.assertFalse(report["raw_archives_identical"])
        self.assertEqual(report["common_content_difference_examples"], ["package/a.json"])
        self.assertEqual(report["common_content_difference_count"], 1)
        self.assertEqual(report["first_only_regular_files_count"], 1)
        self.assertEqual(report["second_only_regular_files_count"], 1)
        self.assertEqual(report["first_only_examples"], ["package/unique-a.json"])
        self.assertEqual(report["second_only_examples"], ["package/unique-b.json"])
        self.assertEqual(report["common_metadata_difference_count"], 2)

    def test_detects_same_files_with_only_tar_metadata_drift(self):
        a = archive([("package/package.json", MANIFEST, 1)])
        b = archive([("package/package.json", MANIFEST, 2)])
        self.write(a, b)
        r = sut.compare(self.first, self.second)
        self.assertFalse(r["raw_archives_identical"])
        self.assertEqual(r["common_content_difference_count"], 0)
        self.assertEqual(r["common_metadata_difference_count"], 1)

    def test_rejects_traversal_and_duplicate_members(self):
        legitimate = [("package/package.json", MANIFEST, 0)]
        for bad in (
            archive(legitimate + [("../escape.json", b"x", 0)]),
            archive(legitimate + [("package/package.json", MANIFEST, 0)]),
            archive(legitimate + [("/absolute", b"x", 0)]),
        ):
            self.write(bad, archive(legitimate))
            with self.assertRaises(sut.AuditError):
                sut.compare(self.first, self.second)

    def test_rejects_corrupt_gzip_and_mismatched_package_identity(self):
        good = archive([("package/package.json", MANIFEST, 0)])
        self.write(b"not a gzip archive", good)
        with self.assertRaises(sut.AuditError):
            sut.compare(self.first, self.second)
        other = archive([("package/package.json", b'{"name":"other","version":"1.0.0"}', 0)])
        self.write(other, good)
        with self.assertRaisesRegex(sut.AuditError, "same declared package identity"):
            sut.compare(self.first, self.second)

    def test_fails_closed_at_compressed_expanded_and_member_limits(self):
        good = archive([("package/package.json", MANIFEST, 0)])
        self.write(good, good)
        with patch.object(sut, "MAX_ARCHIVE_BYTES", 1):
            with self.assertRaisesRegex(sut.AuditError, "compressed byte bound"):
                sut.compare(self.first, self.second)
        with patch.object(sut, "MAX_MEMBER_BYTES", 1):
            with self.assertRaisesRegex(sut.AuditError, "individual expanded member"):
                sut.compare(self.first, self.second)
        with patch.object(sut, "MAX_EXPANDED_BYTES", 1):
            with self.assertRaisesRegex(sut.AuditError, "aggregate expanded member"):
                sut.compare(self.first, self.second)
        with patch.object(sut, "MAX_MEMBERS", 0):
            with self.assertRaisesRegex(sut.AuditError, "member-count"):
                sut.compare(self.first, self.second)


if __name__ == "__main__":
    unittest.main()
