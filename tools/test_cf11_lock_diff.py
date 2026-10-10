"""No-network CF11 lock-difference diagnostic regression checks."""

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

import cf11_lock_diff as diagnostic


def lock():
    return {
        "schema": 2, "roots": ["hl7.fhir.uv.ips@2.0.1"],
        "packages": [
            {"name": "hl7.fhir.uv.ips", "version": "2.0.1", "source": "https://packages.fhir.org/root",
             "sha256": "a" * 64, "dependencies": {"hl7.fhir.r4.core": "4.0.1"}},
            {"name": "hl7.fhir.r4.core", "version": "4.0.1", "source": "https://packages.fhir.org/core",
             "sha256": "b" * 64, "dependencies": {}},
        ],
    }


class LockDiffTests(unittest.TestCase):
    def test_identical_and_order_independent(self):
        a, b = lock(), lock()
        b["packages"].reverse()
        result = diagnostic.compare(a, b)
        self.assertTrue(result["semantic_identical"])
        self.assertEqual(result["differences"], [])

    def test_source_only_does_not_change_semantic_identity(self):
        a, b = lock(), lock()
        b["packages"][0]["source"] = "https://packages2.fhir.org/packages/root"
        result = diagnostic.compare(a, b)
        self.assertTrue(result["semantic_identical"])
        self.assertEqual(result["source_difference_identities"], ["hl7.fhir.uv.ips@2.0.1"])

    def test_same_declared_identity_different_raw_sha_fails(self):
        a, b = lock(), lock()
        b["packages"][0]["sha256"] = "c" * 64
        result = diagnostic.compare(a, b)
        self.assertFalse(result["semantic_identical"])
        self.assertEqual(result["differences"][0]["field"], "sha256")
        self.assertEqual(result["differences"][0]["left"], "a" * 64)
        self.assertEqual(result["differences"][0]["right"], "c" * 64)

    def test_dependency_disagreement_fails(self):
        a, b = lock(), lock()
        b["packages"][0]["dependencies"]["hl7.fhir.r4.core"] = "4.0.2"
        result = diagnostic.compare(a, b)
        self.assertFalse(result["semantic_identical"])
        self.assertEqual(result["differences"][0]["field"], "dependencies")

    def test_root_disagreement_fails(self):
        a, b = lock(), lock()
        b["roots"] = ["another@1.0.0"]
        self.assertEqual(diagnostic.compare(a, b)["differences"][0]["field"], "roots")

    def test_missing_package_fails(self):
        a, b = lock(), lock()
        b["packages"].pop()
        result = diagnostic.compare(a, b)
        self.assertEqual(result["differences"][0]["field"], "package_presence")
        self.assertFalse(result["semantic_identical"])

    def test_duplicate_package_identity_fails_closed(self):
        a = lock()
        a["packages"].append(dict(a["packages"][0]))
        with self.assertRaises(diagnostic.InvalidLock):
            diagnostic.normalize(a)

    def test_invalid_schema_digest_and_source_fail_closed(self):
        for field, value in (("schema", 3), ("schema", True)):
            a = lock()
            a[field] = value
            with self.subTest(field=field, value=value), self.assertRaises(diagnostic.InvalidLock):
                diagnostic.normalize(a)
        for field, value in (("sha256", "not-a-digest"), ("source", "")):
            a = lock()
            a["packages"][0][field] = value
            with self.subTest(field=field), self.assertRaises(diagnostic.InvalidLock):
                diagnostic.normalize(a)

    def test_reject_duplicate_json_object_key(self):
        with tempfile.TemporaryDirectory() as folder:
            f = Path(folder) / "lock.json"
            f.write_text('{"schema":2,"schema":2,"roots":[],"packages":[]}')
            with self.assertRaises(diagnostic.InvalidLock):
                diagnostic.read_lock(f)

    def test_reject_oversized_input(self):
        with tempfile.TemporaryDirectory() as folder:
            f = Path(folder) / "lock.json"
            f.write_bytes(b" " * (diagnostic.MAX_BYTES + 1))
            with self.assertRaises(diagnostic.InvalidLock):
                diagnostic.read_lock(f)

    def test_cli_exit_codes_and_machine_readable_report(self):
        with tempfile.TemporaryDirectory() as folder:
            f, g = Path(folder) / "a.json", Path(folder) / "b.json"
            f.write_text(json.dumps(lock()))
            b = lock()
            b["packages"][0]["sha256"] = "e" * 64
            g.write_text(json.dumps(b))
            stream = io.StringIO()
            with contextlib.redirect_stdout(stream):
                code = diagnostic.main([str(f), str(g)])
            self.assertEqual(code, 1)
            self.assertEqual(json.loads(stream.getvalue())["difference_count"], 1)
            g.write_text(f.read_text())
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(diagnostic.main([str(f), str(g)]), 0)
            g.write_text("{invalid")
            with contextlib.redirect_stdout(stream := io.StringIO()):
                self.assertEqual(diagnostic.main([str(f), str(g)]), 2)
            self.assertFalse(json.loads(stream.getvalue())["semantic_identical"])


if __name__ == "__main__":
    unittest.main()
