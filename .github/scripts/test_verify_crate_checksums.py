#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("verify_crate_checksums.py")
SPEC = importlib.util.spec_from_file_location("verify_crate_checksums_target", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
VERIFY = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = VERIFY
SPEC.loader.exec_module(VERIFY)


def fixture(archive_bytes: bytes = b"crate archive bytes") -> tuple[dict[str, object], list[dict[str, object]], bytes]:
    digest = hashlib.sha256(archive_bytes).hexdigest()
    package_id = f"{VERIFY.CRATES_IO_SOURCE}#dep@1.0.0"
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
                "package_id": package_id,
                "name": "dep",
                "version": "1.0.0",
                "source": VERIFY.CRATES_IO_SOURCE,
                "source_class": "crates.io",
            },
        ],
    }
    lock = [
        {"name": "root", "version": "0.1.0"},
        {
            "name": "dep",
            "version": "1.0.0",
            "source": VERIFY.CRATES_IO_SOURCE,
            "checksum": digest,
        },
    ]
    return inventory, lock, archive_bytes


class CrateChecksumVerificationTests(unittest.TestCase):
    def write_archive(self, cargo_home: Path, content: bytes, registry: str = "index.crates.io-test") -> Path:
        path = cargo_home / "registry" / "cache" / registry / "dep-1.0.0.crate"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def test_fetched_archive_matches_locked_checksum(self) -> None:
        inventory, lock, archive = fixture()
        with tempfile.TemporaryDirectory() as directory:
            cargo_home = Path(directory) / "cargo"
            self.write_archive(cargo_home, archive)
            result = VERIFY.verify(inventory, lock, cargo_home)
        package_id = f"{VERIFY.CRATES_IO_SOURCE}#dep@1.0.0"
        digest = hashlib.sha256(archive).hexdigest()
        self.assertEqual(
            result,
            {"checksums": {package_id: digest}, "ok": True, "package_count": 1, "schema": 1},
        )

    def test_checksum_mismatch_fails_closed(self) -> None:
        inventory, lock, _ = fixture()
        with tempfile.TemporaryDirectory() as directory:
            cargo_home = Path(directory) / "cargo"
            self.write_archive(cargo_home, b"tampered archive")
            with self.assertRaisesRegex(VERIFY.ChecksumError, "checksum mismatch"):
                VERIFY.verify(inventory, lock, cargo_home)

    def test_missing_archive_fails_closed(self) -> None:
        inventory, lock, _ = fixture()
        with tempfile.TemporaryDirectory() as directory:
            cargo_home = Path(directory) / "cargo"
            (cargo_home / "registry" / "cache" / "index.crates.io-test").mkdir(parents=True)
            with self.assertRaisesRegex(VERIFY.ChecksumError, "archive is missing"):
                VERIFY.verify(inventory, lock, cargo_home)

    def test_conflicting_cached_archives_fail_closed(self) -> None:
        inventory, lock, archive = fixture()
        with tempfile.TemporaryDirectory() as directory:
            cargo_home = Path(directory) / "cargo"
            self.write_archive(cargo_home, archive, "index.crates.io-a")
            self.write_archive(cargo_home, b"other bytes", "index.crates.io-b")
            with self.assertRaisesRegex(VERIFY.ChecksumError, "archives disagree"):
                VERIFY.verify(inventory, lock, cargo_home)

    def test_missing_inventory_package_fails_closed(self) -> None:
        inventory, lock, archive = fixture()
        inventory["packages"] = inventory["packages"][:1]
        with tempfile.TemporaryDirectory() as directory:
            cargo_home = Path(directory) / "cargo"
            self.write_archive(cargo_home, archive)
            with self.assertRaisesRegex(VERIFY.ChecksumError, "missing from inventory"):
                VERIFY.verify(inventory, lock, cargo_home)


if __name__ == "__main__":
    unittest.main()
