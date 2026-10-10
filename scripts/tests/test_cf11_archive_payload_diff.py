"""No-network tests for bounded, redacted CF11 TAR archive comparison."""
from __future__ import annotations

import contextlib
import gzip
import importlib.util
import io
import json
import tarfile
import tempfile
import unittest
from pathlib import Path

MODULE = Path(__file__).resolve().parents[1] / "cf11_archive_payload_diff.py"
spec = importlib.util.spec_from_file_location("archive_probe", MODULE)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


def archive(entries: list[tuple[str, bytes]], *, level: int = 6) -> bytes:
    target = io.BytesIO()
    with gzip.GzipFile(fileobj=target, mode="wb", compresslevel=level, mtime=0) as gz:
        with tarfile.open(fileobj=gz, mode="w|") as tar:
            for name, data in entries:
                header = tarfile.TarInfo(name)
                header.size = len(data)
                tar.addfile(header, io.BytesIO(data))
    return target.getvalue()


MANIFEST = json.dumps({"name": "hl7.fhir.uv.ips", "version": "2.0.1",
                       "dependencies": {"hl7.fhir.r4.core": "4.0.1"}}).encode()


class ArchiveDifferenceTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)

    def write(self, name, data):
        path = self.root / name
        path.write_bytes(data)
        return path

    def test_identical_archive_is_success(self):
        data = archive([("package/package.json", MANIFEST), ("package/a.json", b"{}")])
        a, b = self.write("a.tgz", data), self.write("b.tgz", data)
        result = probe.compare_archives(a, b)
        self.assertEqual(result["overall"], "ARCHIVE_BYTES_IDENTICAL")
        self.assertEqual(result["common_changed_files"], 0)
        self.assertEqual(probe.main(["--first", str(a), "--second", str(b)]), 0)

    def test_different_compression_is_not_identical_archive_bytes(self):
        entries = [("package/package.json", MANIFEST), ("package/a.txt", b"x" * 9999)]
        a = self.write("a.tgz", archive(entries, level=1))
        b = self.write("b.tgz", archive(entries, level=9))
        result = probe.compare_archives(a, b)
        self.assertEqual(result["overall"], "ARCHIVE_BYTES_DIFFERENT_CONTENT_IDENTICAL")
        self.assertEqual(result["common_changed_files"], 0)
        self.assertEqual(probe.main(["--first", str(a), "--second", str(b)]), 2)

    def test_identical_payload_but_executable_permission_differs(self):
        def with_mode(mode):
            target = io.BytesIO()
            with gzip.GzipFile(fileobj=target, mode="wb", mtime=0) as gz:
                with tarfile.open(fileobj=gz, mode="w|") as tar:
                    for name, data, permission in [
                        ("package/package.json", MANIFEST, 0o644),
                        ("package/install.sh", b"echo harmless", mode),
                    ]:
                        member = tarfile.TarInfo(name)
                        member.mode = permission
                        member.size = len(data)
                        tar.addfile(member, io.BytesIO(data))
            return target.getvalue()

        a = self.write("regular.tgz", with_mode(0o644))
        b = self.write("executable.tgz", with_mode(0o755))
        result = probe.compare_archives(a, b)
        self.assertEqual(result["overall"], "ARCHIVE_METADATA_DIVERGENCE")
        self.assertEqual(result["common_changed_files"], 0)
        self.assertEqual(result["common_changed_file_metadata"], 1)

    def test_directory_permission_change_is_metadata_divergence(self):
        def with_directory_mode(mode):
            target = io.BytesIO()
            with gzip.GzipFile(fileobj=target, mode="wb", mtime=0) as gz:
                with tarfile.open(fileobj=gz, mode="w|") as tar:
                    directory = tarfile.TarInfo("package/")
                    directory.type = tarfile.DIRTYPE
                    directory.mode = mode
                    tar.addfile(directory)
                    manifest = tarfile.TarInfo("package/package.json")
                    manifest.size = len(MANIFEST)
                    tar.addfile(manifest, io.BytesIO(MANIFEST))
            return target.getvalue()

        a = self.write("restricted.tgz", with_directory_mode(0o700))
        b = self.write("open.tgz", with_directory_mode(0o755))
        result = probe.compare_archives(a, b)
        self.assertEqual(result["overall"], "ARCHIVE_METADATA_DIVERGENCE")
        self.assertEqual(result["common_changed_files"], 0)
        self.assertEqual(result["common_changed_file_metadata"], 0)
        self.assertEqual(result["common_changed_directory_metadata"], 1)
        self.assertEqual(result["directories"], {"first": 1, "second": 1})

    def test_duplicate_directory_headers_are_not_silent(self):
        target = io.BytesIO()
        with gzip.GzipFile(fileobj=target, mode="wb", mtime=0) as gz:
            with tarfile.open(fileobj=gz, mode="w|") as tar:
                for dirname in ("package/", "./package/"):
                    entry = tarfile.TarInfo(dirname)
                    entry.type = tarfile.DIRTYPE
                    tar.addfile(entry)
                manifest = tarfile.TarInfo("package/package.json")
                manifest.size = len(MANIFEST)
                tar.addfile(manifest, io.BytesIO(MANIFEST))
        with self.assertRaisesRegex(probe.InvalidArchive, "duplicate"):
            probe.archive_index(self.write("duplicate-directory.tgz", target.getvalue()))

    def test_real_payload_difference_with_manifest_metadata(self):
        first = MANIFEST
        second = json.dumps({"name": "hl7.fhir.uv.ips", "version": "2.0.1",
                             "dependencies": {"hl7.fhir.r4.core": "4.0.1"},
                             "url": "file:///private/secret",
                             "notForPublication": True}).encode()
        a = self.write("a.tgz", archive([("package/package.json", first),
                                         ("package/private/path.json", b"patient")]))
        b = self.write("b.tgz", archive([("package/package.json", second)]))
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            code = probe.main(["--first", str(a), "--second", str(b)])
        result = json.loads(output.getvalue())
        self.assertEqual(code, 2)
        self.assertEqual(result["overall"], "ARCHIVE_CONTENT_DIVERGENCE")
        self.assertEqual(result["first_only_files"], 1)
        self.assertEqual(result["common_changed_files"], 1)
        self.assertTrue(result["manifest_name_version_equal"])
        self.assertTrue(result["manifest_dependencies_equal"])
        self.assertEqual(result["manifest_selected_changed_fields"], ["notForPublication", "url"])
        self.assertNotIn("private", output.getvalue())
        self.assertNotIn(self.directory.name, output.getvalue())
        self.assertNotIn("patient", output.getvalue())

    def test_duplicate_exact_or_normalized_member_is_rejected(self):
        for alias in ("package/package.json", "./package/package.json"):
            with self.subTest(alias=alias):
                data = archive([("package/package.json", MANIFEST), (alias, MANIFEST)])
                with self.assertRaisesRegex(probe.InvalidArchive, "duplicate"):
                    probe.archive_index(self.write("dup.tgz", data))

    def test_missing_manifest_is_rejected(self):
        with self.assertRaisesRegex(probe.InvalidArchive, "missing"):
            probe.archive_index(self.write("missing.tgz", archive([("other.txt", b"x")])))

    def test_bad_gzip_and_oversize_return_generic_error(self):
        for name, data in (("bad.tgz", b"not gzip"), ("large.tgz", b"X" * 80)):
            with self.subTest(name=name):
                path = self.write(name, data)
                old = probe.MAX_ARCHIVE_BYTES
                probe.MAX_ARCHIVE_BYTES = 60
                try:
                    output = io.StringIO()
                    with contextlib.redirect_stdout(output):
                        code = probe.main(["--first", str(path), "--second", str(path)])
                finally:
                    probe.MAX_ARCHIVE_BYTES = old
                self.assertEqual(code, 4)
                self.assertEqual(json.loads(output.getvalue())["overall"], "INVALID_ARCHIVE")
                self.assertNotIn(self.directory.name, output.getvalue())

    def test_invalid_unsafe_entry_path(self):
        for name in ("../secret", "/tmp/secret", "package/../secret"):
            with self.subTest(name=name):
                with self.assertRaises(probe.InvalidArchive):
                    probe.canonical_member_name(name)

    def test_bounded_inflated_bytes(self):
        data = archive([("package/package.json", MANIFEST), ("package/big", b"x" * 60000)])
        old = probe.MAX_DECOMPRESSED_BYTES
        probe.MAX_DECOMPRESSED_BYTES = 10000
        try:
            with self.assertRaises(probe.InvalidArchive):
                probe.archive_index(self.write("large-expanded.tgz", data))
        finally:
            probe.MAX_DECOMPRESSED_BYTES = old

    def test_member_count_limit(self):
        data = archive([("package/package.json", MANIFEST), ("package/other", b"x")])
        old = probe.MAX_MEMBERS
        probe.MAX_MEMBERS = 1
        try:
            with self.assertRaisesRegex(probe.InvalidArchive, "member count"):
                probe.archive_index(self.write("too-many.tgz", data))
        finally:
            probe.MAX_MEMBERS = old

    def test_truncated_archive_is_invalid(self):
        data = archive([("package/package.json", MANIFEST), ("package/other", b"x")])
        data = data[:-7]
        with self.assertRaises((EOFError, OSError, gzip.BadGzipFile)):
            probe.archive_index(self.write("truncated.tgz", data))

    def test_manifest_duplicate_json_keys_are_rejected(self):
        ambiguous = b'{"name":"hl7.fhir.uv.ips","name":"acme.malicious","version":"2.0.1"}'
        data = archive([("package/package.json", ambiguous)])
        with self.assertRaisesRegex(probe.InvalidArchive, "duplicate"):
            probe.archive_index(self.write("duplicate-keys.tgz", data))

    def test_manifest_missing_identity_rejected(self):
        data = archive([("package/package.json", b'{"name":"acme.example"}')])
        with self.assertRaisesRegex(probe.InvalidArchive, "identity"):
            probe.archive_index(self.write("missing-identity.tgz", data))

    def test_missing_versus_explicit_null_is_not_equal(self):
        base = {"name": "acme.example", "version": "1.0.0"}
        a = self.write("a.tgz", archive([
            ("package/package.json", json.dumps(base).encode())]))
        b = self.write("b.tgz", archive([
            ("package/package.json", json.dumps({**base, "notForPublication": None}).encode())]))
        report = probe.compare_archives(a, b)
        self.assertEqual(report["overall"], "ARCHIVE_CONTENT_DIVERGENCE")
        self.assertEqual(report["manifest_selected_changed_fields"], ["notForPublication"])

    def test_links_are_not_followed_or_interpreted(self):
        target = io.BytesIO()
        with gzip.GzipFile(fileobj=target, mode="wb", mtime=0) as gz:
            with tarfile.open(fileobj=gz, mode="w|") as tar:
                header = tarfile.TarInfo("package/package.json")
                header.size = len(MANIFEST)
                tar.addfile(header, io.BytesIO(MANIFEST))
                link = tarfile.TarInfo("package/secret")
                link.type = tarfile.SYMTYPE
                link.linkname = "/private/secret"
                tar.addfile(link)
        with self.assertRaisesRegex(probe.InvalidArchive, "unsupported TAR"):
            probe.archive_index(self.write("link.tgz", target.getvalue()))

    def test_manifest_limit(self):
        data = archive([("package/package.json", MANIFEST)])
        old = probe.MAX_MANIFEST_BYTES
        probe.MAX_MANIFEST_BYTES = 5
        try:
            with self.assertRaisesRegex(probe.InvalidArchive, "manifest byte"):
                probe.archive_index(self.write("oversize-manifest.tgz", data))
        finally:
            probe.MAX_MANIFEST_BYTES = old


if __name__ == "__main__":
    unittest.main()
