#!/usr/bin/env python3
"""Read-only diagnostic for exact-version FHIR archive identity across registry mirrors.

Not a qualification oracle or replacement for the mandatory CF11 CI assertion.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

PRIMARY = "https://packages.fhir.org"
SECONDARY = "https://packages2.fhir.org/web"
MAX_LOCK_BYTES = 16 * 1024 * 1024
MAX_ARCHIVE_BYTES = 128 * 1024 * 1024
MAX_COMPARABLE_PACKAGES = 4096
MAX_NETWORK_PROBE_PACKAGES = 16
PACKAGE_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9.-]{0,199}\Z")
VERSION = re.compile(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?\Z")


class InvalidInput(ValueError):
    """The request cannot be safely interpreted as an exact package identity."""


def no_duplicate_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise InvalidInput("lockfile contains duplicate object keys")
        result[key] = value
    return result


def load_lock(path: Path) -> dict:
    with path.open("rb") as handle:
        data = handle.read(MAX_LOCK_BYTES + 1)
    if len(data) > MAX_LOCK_BYTES:
        raise InvalidInput("lockfile exceeds 16 MiB")
    try:
        lock = json.loads(data, object_pairs_hook=no_duplicate_keys)
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        raise InvalidInput("lockfile is not valid JSON") from exc
    if (not isinstance(lock, dict)
            or type(lock.get("schema")) is not int
            or lock["schema"] not in (1, 2)):
        raise InvalidInput("unsupported lockfile schema")
    packages = lock.get("packages")
    if not isinstance(packages, list) or not packages:
        raise InvalidInput("lockfile has no packages")
    unique = set()
    for package in packages:
        if not isinstance(package, dict):
            raise InvalidInput("lockfile package is not an object")
        name, version, digest = (
            package.get("name"), package.get("version"), package.get("sha256")
        )
        validate_identity(name, version)
        if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise InvalidInput("package digest is not lowercase SHA-256")
        identity = (name, version)
        if identity in unique:
            raise InvalidInput("duplicate exact package identity in lockfile")
        unique.add(identity)
    return lock


def load_packages(path: Path) -> list[dict]:
    return load_lock(path)["packages"]


def validate_identity(name: str, version: str) -> None:
    if (not isinstance(name, str) or not PACKAGE_NAME.fullmatch(name)
            or ".." in name or name.endswith(".")
            or not isinstance(version, str) or not VERSION.fullmatch(version)):
        raise InvalidInput("invalid exact package name/version")


def source_kind(source: str) -> str:
    """Do not expose raw provenance URLs: local sources can contain host paths."""
    if source.startswith(PRIMARY + "/"):
        return "primary"
    if source.startswith(SECONDARY + "/"):
        return "secondary"
    return "other-redacted"


def lock_projection(path: Path) -> dict:
    """Bounded semantic projection, independent of source-URL disclosure."""
    lock = load_lock(path)
    roots = lock.get("roots")
    if not isinstance(roots, list) or any(not isinstance(r, str) for r in roots):
        raise InvalidInput("comparison requires a valid roots list")
    packages = lock["packages"]
    if len(packages) > MAX_COMPARABLE_PACKAGES:
        raise InvalidInput("too many package identities to compare")
    result = {}
    for package in packages:
        dependencies = package.get("dependencies")
        source = package.get("source")
        if (not isinstance(dependencies, dict)
                or any(not isinstance(name, str) or not isinstance(value, str)
                       for name, value in dependencies.items())):
            raise InvalidInput("comparison requires string dependencies")
        if not isinstance(source, str) or not source:
            raise InvalidInput("comparison requires source provenance")
        key = (package["name"], package["version"])
        result[key] = {
            "sha256": package["sha256"],
            "dependencies": dependencies,
            "source": source,
        }
    return {"schema": lock["schema"], "roots": sorted(roots), "packages": result}


def compare_locks(first_path: Path, second_path: Path) -> dict:
    """Compare both exact lock observations, preserving any real mismatches."""
    first, second = lock_projection(first_path), lock_projection(second_path)
    identities = sorted(first["packages"].keys() | second["packages"].keys())
    differences = []
    for name, version in identities:
        a = first["packages"].get((name, version))
        b = second["packages"].get((name, version))
        if a == b:
            continue
        differences.append({
            "name": name,
            "version": version,
            "present_in_first": a is not None,
            "present_in_second": b is not None,
            "sha256_first": a["sha256"] if a else None,
            "sha256_second": b["sha256"] if b else None,
            "digest_equal": a["sha256"] == b["sha256"] if a and b else None,
            "dependencies_equal": (
                a["dependencies"] == b["dependencies"] if a and b else None
            ),
            "source_equal": a["source"] == b["source"] if a and b else None,
            "source_kind_first": source_kind(a["source"]) if a else None,
            "source_kind_second": source_kind(b["source"]) if b else None,
        })
    same_package_set = first["packages"].keys() == second["packages"].keys()
    semantic_equal = (
        same_package_set
        and first["schema"] == second["schema"]
        and first["roots"] == second["roots"]
        and all(
            first["packages"][key]["sha256"] == second["packages"][key]["sha256"]
            and first["packages"][key]["dependencies"]
            == second["packages"][key]["dependencies"]
            for key in identities
        )
    )
    provenance_equal = (
        same_package_set
        and all(first["packages"][key]["source"] == second["packages"][key]["source"]
                for key in identities)
    )
    overall = ("IDENTICAL" if semantic_equal and provenance_equal
               else "PROVENANCE_ONLY_DIFFERENCE" if semantic_equal
               else "SEMANTIC_LOCK_DIFFERENCE")
    return {
        "schema": "commandf.cf11-lock-comparison/v1",
        "overall": overall,
        "semantic_lock_identity_identical": semantic_equal,
        "transport_provenance_identical": provenance_equal,
        "lock_schema_equal": first["schema"] == second["schema"],
        "roots_equal": first["roots"] == second["roots"],
        "package_counts": {"first": len(first["packages"]),
                           "second": len(second["packages"])},
        "differences": differences,
    }


def fetch_digest(url: str, max_archive_bytes: int = MAX_ARCHIVE_BYTES) -> dict:
    """Bounded, direct curl fetch using platform TLS; no redirect or disk retention."""
    if not (url.startswith(f"{PRIMARY}/") or url.startswith(f"{SECONDARY}/")):
        raise InvalidInput("unexpected registry host")
    with tempfile.TemporaryDirectory(prefix="cf11-registry-probe-") as tmp:
        target = Path(tmp) / "archive.tgz"
        command = [
            "curl", "-q", "--silent", "--max-time", "30",
            "--max-filesize", str(max_archive_bytes),
            "--proto", "=https", "--noproxy", "*",
            "--output", str(target), "--write-out", "%{http_code}", url,
        ]
        try:
            result = subprocess.run(
                command, capture_output=True, timeout=35, check=False
            )
        except (OSError, subprocess.TimeoutExpired):
            return {"state": "UNAVAILABLE", "reason": "TRANSPORT"}
        if result.returncode == 63:
            return {"state": "UNAVAILABLE", "reason": "ARCHIVE_LIMIT"}
        if result.returncode != 0:
            return {"state": "UNAVAILABLE", "reason": "TRANSPORT"}
        try:
            status = int(result.stdout.strip())
        except ValueError:
            return {"state": "UNAVAILABLE", "reason": "HTTP_STATUS_UNKNOWN"}
        if status != 200:
            return {"state": "UNAVAILABLE", "http_status": status}
        try:
            size = target.stat().st_size
            if size > max_archive_bytes:
                return {"state": "UNAVAILABLE", "reason": "ARCHIVE_LIMIT"}
            digest = hashlib.sha256()
            with target.open("rb") as stream:
                if stream.read(2) != b"\x1f\x8b":
                    return {"state": "UNAVAILABLE", "reason": "NOT_GZIP"}
                stream.seek(0)
                for block in iter(lambda: stream.read(1024 * 1024), b""):
                    digest.update(block)
        except OSError:
            return {"state": "UNAVAILABLE", "reason": "ARCHIVE_IO"}
        return {"state": "AVAILABLE", "sha256": digest.hexdigest(), "bytes": size}


def check_package(package: dict, fetch=fetch_digest) -> dict:
    name, version = package["name"], package["version"]
    validate_identity(name, version)
    endpoints = (
        ("primary", f"{PRIMARY}/{name}/{version}"),
        ("secondary", f"{SECONDARY}/{name}-{version}.tgz"),
    )
    responses = {label: fetch(url) for label, url in endpoints}
    available = [item for item in responses.values() if item["state"] == "AVAILABLE"]
    pin = package["sha256"]
    if len(available) == 2 and available[0]["sha256"] != available[1]["sha256"]:
        classification = "SOURCE_IDENTITY_CONFLICT"
    elif any(item["sha256"] != pin for item in available):
        classification = "LOCK_DIGEST_DISAGREEMENT"
    elif len(available) != 2:
        classification = "INCOMPLETE_SOURCE_OBSERVATION"
    else:
        classification = "SOURCE_BYTES_IDENTICAL"
    return {
        "name": name, "version": version,
        "classification": classification, "expected_lock_sha256": pin,
        "lock_sha256_matches": {
            label: item["sha256"] == pin if item["state"] == "AVAILABLE" else None
            for label, item in responses.items()
        },
        "sources": responses,
    }


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lock", type=Path, required=True,
                        help="locally trusted, pre-existing commandf.lock")
    parser.add_argument("--compare-lock", type=Path,
                        help="compare two saved local lockfiles without network access")
    parser.add_argument("--all", action="store_true",
                        help="probe all exact identities in lock (may download many archives)")
    parser.add_argument("--name", help="exact package name; required unless --all")
    parser.add_argument("--version", help="exact version; required unless --all")
    args = parser.parse_args(argv)
    try:
        if args.compare_lock is not None:
            if args.all or args.name or args.version:
                raise InvalidInput("--compare-lock excludes network probe selectors")
            report = compare_locks(args.lock, args.compare_lock)
            print(json.dumps(report, sort_keys=True, indent=2))
            return 0 if report["overall"] == "IDENTICAL" else 2
        if args.all and (args.name or args.version):
            raise InvalidInput("--all cannot be combined with --name or --version")
        if not args.all:
            validate_identity(args.name, args.version)
        packages = load_packages(args.lock)
        if args.all and len(packages) > MAX_NETWORK_PROBE_PACKAGES:
            raise InvalidInput(
                "network --all exceeds 16 packages; select an exact name/version"
            )
        if not args.all:
            packages = [p for p in packages
                        if (p["name"], p["version"]) == (args.name, args.version)]
            if not packages:
                raise InvalidInput("exact package not present in lock")
        # Lock ordering is not an input to diagnosis or network request ordering.
        packages.sort(key=lambda package: (package["name"], package["version"]))
        results = [check_package(p) for p in packages]
        worst = (
            "SOURCE_IDENTITY_CONFLICT" if any(
                r["classification"] == "SOURCE_IDENTITY_CONFLICT" for r in results
            ) else "LOCK_DIGEST_DISAGREEMENT" if any(
                r["classification"] == "LOCK_DIGEST_DISAGREEMENT" for r in results
            ) else "INCOMPLETE_SOURCE_OBSERVATION" if any(
                r["classification"] == "INCOMPLETE_SOURCE_OBSERVATION" for r in results
            ) else "SOURCE_BYTES_IDENTICAL"
        )
        print(json.dumps({"schema": "commandf.cf11-registry-probe/v1",
                          "overall": worst, "packages": results}, sort_keys=True, indent=2))
        return {"SOURCE_BYTES_IDENTICAL": 0,
                "SOURCE_IDENTITY_CONFLICT": 2,
                "LOCK_DIGEST_DISAGREEMENT": 2,
                "INCOMPLETE_SOURCE_OBSERVATION": 3}[worst]
    except (InvalidInput, OSError) as exc:
        # Do not emit OS errors, which may expose local host paths.
        print(json.dumps({"schema": "commandf.cf11-registry-probe/v1",
                          "overall": "INVALID_INPUT",
                          "reason": str(exc) if isinstance(exc, InvalidInput)
                          else "LOCKFILE_IO"}), file=sys.stdout)
        return 4


if __name__ == "__main__":
    raise SystemExit(main())
