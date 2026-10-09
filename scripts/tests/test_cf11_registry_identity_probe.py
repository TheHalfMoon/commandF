"""Offline positive/negative tests for the non-authoritative CF11 registry probe."""
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock


MODULE = Path(__file__).resolve().parents[1] / "cf11_registry_identity_probe.py"
spec = importlib.util.spec_from_file_location("cf11_probe", MODULE)
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class IdentityProbeTests(unittest.TestCase):
    def setUp(self):
        self.p = {"name": "hl7.fhir.uv.ips", "version": "2.0.1", "sha256": "a" * 64}

    def test_equal_mirror_bytes_and_matching_lock(self):
        got = probe.check_package(self.p, lambda _: {
            "state": "AVAILABLE", "sha256": "a" * 64, "bytes": 12})
        self.assertEqual(got["classification"], "SOURCE_BYTES_IDENTICAL")
        self.assertEqual(got["lock_sha256_matches"], {"primary": True, "secondary": True})

    def test_conflicting_mirror_bytes_fail_closed(self):
        def fake(url):
            digest = "b" * 64 if url.startswith(probe.SECONDARY) else "a" * 64
            return {"state": "AVAILABLE", "sha256": digest, "bytes": 12}
        got = probe.check_package(self.p, fake)
        self.assertEqual(got["classification"], "SOURCE_IDENTITY_CONFLICT")
        self.assertFalse(got["lock_sha256_matches"]["secondary"])

    def test_identical_mirrors_disagreeing_with_pinned_lock_fail_closed(self):
        got = probe.check_package(self.p, lambda _: {
            "state": "AVAILABLE", "sha256": "b" * 64, "bytes": 12})
        self.assertEqual(got["classification"], "LOCK_DIGEST_DISAGREEMENT")
        self.assertFalse(got["lock_sha256_matches"]["primary"])

    def test_one_mirror_disagreeing_with_lock_still_fails_closed(self):
        def fake(url):
            if url.startswith(probe.SECONDARY):
                return {"state": "UNAVAILABLE", "http_status": 503}
            return {"state": "AVAILABLE", "sha256": "b" * 64, "bytes": 12}
        self.assertEqual(probe.check_package(self.p, fake)["classification"],
                         "LOCK_DIGEST_DISAGREEMENT")

    def test_unavailable_mirror_not_treated_as_identical(self):
        def fake(url):
            if url.startswith(probe.SECONDARY):
                return {"state": "UNAVAILABLE", "http_status": 503}
            return {"state": "AVAILABLE", "sha256": "a" * 64, "bytes": 12}
        got = probe.check_package(self.p, fake)
        self.assertEqual(got["classification"], "INCOMPLETE_SOURCE_OBSERVATION")
        self.assertIsNone(got["lock_sha256_matches"]["secondary"])

    def test_invalid_package_and_path_injection(self):
        for name, version in [("../secrets", "2.0.1"), ("pkg", "../evil"),
                              ("pkg", "latest"), ("pkg/evil", "1.0.0")]:
            with self.subTest(name=name, version=version):
                with self.assertRaises(probe.InvalidInput):
                    probe.validate_identity(name, version)

    def test_duplicate_identity_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "commandf.lock"
            path.write_text(json.dumps({"schema": 2, "packages": [self.p, self.p]}))
            with self.assertRaisesRegex(probe.InvalidInput, "duplicate exact"):
                probe.load_packages(path)

    def test_duplicate_json_keys_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "commandf.lock"
            path.write_text('{"schema":2,"schema":2,"packages":[]}')
            with self.assertRaisesRegex(probe.InvalidInput, "duplicate object"):
                probe.load_packages(path)

    def test_bounded_input_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "commandf.lock"
            path.write_bytes(b"X" * (probe.MAX_LOCK_BYTES + 1))
            with self.assertRaisesRegex(probe.InvalidInput, "16 MiB"):
                probe.load_packages(path)

    def test_fetch_redirect_and_invalid_archive_fail_closed(self):
        def fake_run(command, **_):
            target = Path(command[command.index("--output") + 1])
            target.write_bytes(b"\x1f\x8btest")
            return mock.Mock(returncode=0, stdout=b"302")
        with mock.patch.object(probe.subprocess, "run", side_effect=fake_run):
            self.assertEqual(probe.fetch_digest(
                "https://packages.fhir.org/pkg/1.0.0")["http_status"], 302)

        def fake_invalid(command, **_):
            target = Path(command[command.index("--output") + 1])
            target.write_bytes(b"not gzip")
            return mock.Mock(returncode=0, stdout=b"200")
        with mock.patch.object(probe.subprocess, "run", side_effect=fake_invalid):
            self.assertEqual(probe.fetch_digest(
                "https://packages.fhir.org/pkg/1.0.0")["reason"], "NOT_GZIP")

    def test_fetch_bounded_and_positive_bytes(self):
        def fake_run(command, **_):
            target = Path(command[command.index("--output") + 1])
            target.write_bytes(b"\x1f\x8bdata")
            return mock.Mock(returncode=0, stdout=b"200")
        with mock.patch.object(probe.subprocess, "run", side_effect=fake_run):
            self.assertEqual(probe.fetch_digest(
                "https://packages.fhir.org/pkg/1.0.0", 4)["reason"], "ARCHIVE_LIMIT")
            result = probe.fetch_digest("https://packages.fhir.org/pkg/1.0.0")
            self.assertEqual(result["state"], "AVAILABLE")
            self.assertEqual(result["sha256"], __import__("hashlib").sha256(
                b"\x1f\x8bdata").hexdigest())

    def test_curl_does_not_load_config_follow_redirect_or_use_proxy(self):
        def fake_run(command, **_):
            self.assertEqual(command[1], "-q")
            self.assertNotIn("--location", command)
            self.assertEqual(command[command.index("--noproxy") + 1], "*")
            return mock.Mock(returncode=7, stdout=b"000")
        with mock.patch.object(probe.subprocess, "run", side_effect=fake_run):
            self.assertEqual(probe.fetch_digest(
                "https://packages.fhir.org/pkg/1.0.0")["reason"], "TRANSPORT")

    def test_result_exit_codes(self):
        # Assert fail-closed and clean CLI errors without external network.
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "commandf.lock"
            path.write_text(json.dumps({"schema": 2, "packages": [self.p]}))
            with mock.patch.object(probe, "check_package", return_value={
                "classification": "SOURCE_IDENTITY_CONFLICT"}):
                self.assertEqual(probe.main(["--lock", str(path), "--all"]), 2)
            with mock.patch.object(probe, "check_package", return_value={
                "classification": "INCOMPLETE_SOURCE_OBSERVATION"}):
                self.assertEqual(probe.main(["--lock", str(path), "--all"]), 3)
            with mock.patch.object(probe, "check_package", return_value={
                "classification": "LOCK_DIGEST_DISAGREEMENT"}):
                self.assertEqual(probe.main(["--lock", str(path), "--all"]), 2)
            self.assertEqual(probe.main(["--lock", str(path), "--name",
                                         "bad/name", "--version", "1.0.0"]), 4)


if __name__ == "__main__":
    unittest.main()
