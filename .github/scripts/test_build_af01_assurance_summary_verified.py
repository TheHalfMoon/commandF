#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("build_af01_assurance_summary_verified.py")
SPEC = importlib.util.spec_from_file_location("af01_assurance_verified", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
VERIFIED = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = VERIFIED
SPEC.loader.exec_module(VERIFIED)


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


class VerifiedAssuranceSummaryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        base = Path(self.temp.name)
        self.root = base / "repo"
        self.evidence = base / "evidence"
        self.cache = base / "cache-home"
        self.root.mkdir()
        self.evidence.mkdir()

        self.archive = b"independently fetched crate archive bytes"
        self.digest = hashlib.sha256(self.archive).hexdigest()
        self.package_id = f"{VERIFIED.VERIFY.CRATES_IO_SOURCE}#dep@1.0.0"

        (self.root / "Cargo.lock").write_text(
            "version = 4\n\n"
            "[[package]]\n"
            "name = \"root\"\n"
            "version = \"0.1.0\"\n\n"
            "[[package]]\n"
            "name = \"dep\"\n"
            "version = \"1.0.0\"\n"
            f"source = \"{VERIFIED.VERIFY.CRATES_IO_SOURCE}\"\n"
            f"checksum = \"{self.digest}\"\n",
            encoding="utf-8",
        )

        inventory = {
            "schema": 2,
            "ok": True,
            "packages": [
                {
                    "package_id": "path+file:///workspace/root#0.1.0",
                    "name": "root",
                    "version": "0.1.0",
                    "source": None,
                    "source_class": "workspace",
                },
                {
                    "package_id": self.package_id,
                    "name": "dep",
                    "version": "1.0.0",
                    "source": VERIFIED.VERIFY.CRATES_IO_SOURCE,
                    "source_class": "crates.io",
                },
            ],
        }
        write_json(self.evidence / "dependency-inventory.json", inventory)

        archive_path = self.cache / "registry" / "cache" / "index.crates.io-test" / "dep-1.0.0.crate"
        archive_path.parent.mkdir(parents=True)
        archive_path.write_bytes(self.archive)

        checksum_evidence = VERIFIED.VERIFY.verify(
            inventory,
            VERIFIED.VERIFY.load_lock(self.root / "Cargo.lock"),
            self.cache,
        )
        checksum_path = self.evidence / VERIFIED.CHECKSUM_EVIDENCE
        write_json(checksum_path, checksum_evidence)
        checksum_sha = hashlib.sha256(checksum_path.read_bytes()).hexdigest()

        write_json(
            self.evidence / "dependency-inventory-proof.json",
            {
                "schema": 1,
                "head_sha": "a" * 40,
                "fetch_command": VERIFIED.FETCH_COMMAND,
                "crate_checksum_package_count": 1,
                "crate_checksums_sha256": checksum_sha,
            },
        )

    def tearDown(self) -> None:
        self.temp.cleanup()

    def test_exact_archive_bytes_are_bound_into_final_proof(self) -> None:
        checksum_sha, count = VERIFIED.validate_checksum_binding(
            self.root, self.evidence, self.cache
        )
        self.assertEqual(count, 1)
        self.assertEqual(
            checksum_sha,
            hashlib.sha256((self.evidence / VERIFIED.CHECKSUM_EVIDENCE).read_bytes()).hexdigest(),
        )

    def test_tampered_recorded_checksum_evidence_fails_closed(self) -> None:
        path = self.evidence / VERIFIED.CHECKSUM_EVIDENCE
        value = json.loads(path.read_text(encoding="utf-8"))
        value["checksums"][self.package_id] = "0" * 64
        write_json(path, value)
        with self.assertRaisesRegex(
            VERIFIED.BASE.AssuranceError, "does not match independently reverified archive bytes"
        ):
            VERIFIED.validate_checksum_binding(self.root, self.evidence, self.cache)

    def test_tampered_fetched_archive_fails_closed(self) -> None:
        archive_path = self.cache / "registry" / "cache" / "index.crates.io-test" / "dep-1.0.0.crate"
        archive_path.write_bytes(b"tampered")
        with self.assertRaisesRegex(
            VERIFIED.BASE.AssuranceError, "fetched crate checksum verification failed"
        ):
            VERIFIED.validate_checksum_binding(self.root, self.evidence, self.cache)

    def test_dependency_proof_must_bind_checksum_evidence_digest(self) -> None:
        proof_path = self.evidence / "dependency-inventory-proof.json"
        proof = json.loads(proof_path.read_text(encoding="utf-8"))
        proof["crate_checksums_sha256"] = "0" * 64
        write_json(proof_path, proof)
        with self.assertRaisesRegex(
            VERIFIED.BASE.AssuranceError, "does not bind exact crate checksum evidence"
        ):
            VERIFIED.validate_checksum_binding(self.root, self.evidence, self.cache)

    def test_dependency_proof_must_bind_fetch_command(self) -> None:
        proof_path = self.evidence / "dependency-inventory-proof.json"
        proof = json.loads(proof_path.read_text(encoding="utf-8"))
        proof["fetch_command"] = ["cargo", "fetch"]
        write_json(proof_path, proof)
        with self.assertRaisesRegex(
            VERIFIED.BASE.AssuranceError, "does not bind exact crate checksum evidence"
        ):
            VERIFIED.validate_checksum_binding(self.root, self.evidence, self.cache)


if __name__ == "__main__":
    unittest.main()
