#!/usr/bin/env python3
"""Verify fetched crates.io archive bytes against exact Cargo.lock checksums."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import tomllib
from pathlib import Path
from typing import Any

CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
HEX64 = re.compile(r"^[0-9a-f]{64}$")


class ChecksumError(ValueError):
    """Raised when fetched crate bytes cannot prove the locked checksum."""


def require_string(value: object, message: str) -> str:
    if not isinstance(value, str) or not value:
        raise ChecksumError(message)
    return value


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ChecksumError(f"invalid dependency inventory: {error}") from error
    if not isinstance(value, dict):
        raise ChecksumError("dependency inventory root is not an object")
    return value


def load_lock(path: Path) -> list[dict[str, Any]]:
    try:
        value = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
        raise ChecksumError(f"invalid Cargo.lock: {error}") from error
    packages = value.get("package")
    if not isinstance(packages, list) or not all(isinstance(item, dict) for item in packages):
        raise ChecksumError("Cargo.lock package records are missing or malformed")
    return packages


def archive_candidates(cargo_home: Path, name: str, version: str) -> list[Path]:
    filename = f"{name}-{version}.crate"
    cache_root = cargo_home / "registry" / "cache"
    if not cache_root.is_dir():
        raise ChecksumError(f"Cargo registry cache is missing: {cache_root}")
    return sorted(path for path in cache_root.glob(f"*/{filename}") if path.is_file())


def verify(inventory: dict[str, Any], lock_packages: list[dict[str, Any]], cargo_home: Path) -> dict[str, object]:
    packages = inventory.get("packages")
    if inventory.get("schema") != 2 or inventory.get("ok") is not True or not isinstance(packages, list):
        raise ChecksumError("dependency inventory is not a successful schema-2 inventory")

    lock_by_identity: dict[tuple[str, str, str], str] = {}
    for index, package in enumerate(lock_packages):
        name = require_string(package.get("name"), f"Cargo.lock package {index} has invalid name")
        version = require_string(package.get("version"), f"Cargo.lock package {index} has invalid version")
        source = package.get("source")
        if source != CRATES_IO_SOURCE:
            continue
        checksum = package.get("checksum")
        if not isinstance(checksum, str) or HEX64.fullmatch(checksum) is None:
            raise ChecksumError(f"Cargo.lock crates.io package {name}@{version} has invalid checksum")
        identity = (name, version, source)
        if identity in lock_by_identity:
            raise ChecksumError(f"duplicate crates.io Cargo.lock identity: {name}@{version}")
        lock_by_identity[identity] = checksum

    checksums: dict[str, str] = {}
    seen_lock_identities: set[tuple[str, str, str]] = set()
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            raise ChecksumError(f"dependency inventory package {index} is not an object")
        if package.get("source_class") != "crates.io":
            continue
        package_id = require_string(
            package.get("package_id"), f"dependency inventory package {index} has invalid package id"
        )
        name = require_string(package.get("name"), f"dependency inventory package {index} has invalid name")
        version = require_string(
            package.get("version"), f"dependency inventory package {index} has invalid version"
        )
        source = package.get("source")
        if source != CRATES_IO_SOURCE:
            raise ChecksumError(f"inventory crates.io package {package_id} has inconsistent source")
        identity = (name, version, source)
        expected = lock_by_identity.get(identity)
        if expected is None:
            raise ChecksumError(f"inventory crates.io package is absent from Cargo.lock: {name}@{version}")
        if identity in seen_lock_identities:
            raise ChecksumError(f"duplicate crates.io inventory identity: {name}@{version}")
        seen_lock_identities.add(identity)

        archives = archive_candidates(cargo_home, name, version)
        if not archives:
            raise ChecksumError(f"fetched crate archive is missing: {name}@{version}")
        digests = sorted({sha256_file(path) for path in archives})
        if len(digests) != 1:
            raise ChecksumError(f"fetched crate archives disagree: {name}@{version}")
        actual = digests[0]
        if actual != expected:
            raise ChecksumError(
                f"fetched crate checksum mismatch for {name}@{version}: expected {expected}, found {actual}"
            )
        checksums[package_id] = actual

    if seen_lock_identities != set(lock_by_identity):
        missing = sorted(f"{name}@{version}" for name, version, _ in set(lock_by_identity) - seen_lock_identities)
        raise ChecksumError(f"Cargo.lock crates.io packages missing from inventory: {missing[:1]}")

    ordered = dict(sorted(checksums.items()))
    return {
        "checksums": ordered,
        "ok": True,
        "package_count": len(ordered),
        "schema": 1,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, required=True)
    parser.add_argument("--cargo-lock", type=Path, default=Path("Cargo.lock"))
    parser.add_argument(
        "--cargo-home",
        type=Path,
        default=Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))),
    )
    args = parser.parse_args()
    try:
        result = verify(load_json(args.inventory), load_lock(args.cargo_lock), args.cargo_home)
    except ChecksumError as error:
        print(json.dumps({"error": str(error), "ok": False}, sort_keys=True))
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
