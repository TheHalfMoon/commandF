#!/usr/bin/env python3
"""Offline CF11 lockfile identity diagnostic; does not replace protected CI."""

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

MAX_BYTES = 4 * 1024 * 1024
MAX_PACKAGES = 10000
MAX_DIFFERENCES = 100
SHA256 = re.compile(r"^[0-9a-f]{64}$")


class InvalidLock(ValueError):
    """An input cannot be trusted as a complete CF11 lockfile."""


def no_duplicate_keys(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise InvalidLock("duplicate JSON object key")
        obj[key] = value
    return obj


def required_string(value, label):
    if not isinstance(value, str) or not value or len(value) > 4096:
        raise InvalidLock(f"invalid {label}: expected nonempty bounded string")
    return value


def read_lock(path):
    try:
        if path.stat().st_size > MAX_BYTES:
            raise InvalidLock("lockfile exceeds 4 MiB input budget")
        with path.open("rb") as stream:
            content = stream.read(MAX_BYTES + 1)
        if len(content) > MAX_BYTES:
            raise InvalidLock("lockfile exceeds 4 MiB input budget")
        return json.loads(content.decode("utf-8"), object_pairs_hook=no_duplicate_keys)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise InvalidLock(f"cannot load lockfile: {type(error).__name__}") from error


def normalize(lock):
    """Match the CF11 workflow's semantic projection, rejecting malformed input."""
    if not isinstance(lock, dict) or type(lock.get("schema")) is not int:
        raise InvalidLock("invalid schema")
    if lock["schema"] != 2:
        raise InvalidLock("unsupported lockfile schema")
    roots = lock.get("roots")
    packages = lock.get("packages")
    if not isinstance(roots, list) or not isinstance(packages, list):
        raise InvalidLock("missing roots or packages list")
    if len(roots) > MAX_PACKAGES or len(packages) > MAX_PACKAGES:
        raise InvalidLock("lockfile collection exceeds budget")
    roots = [required_string(root, "root") for root in roots]
    if len(set(roots)) != len(roots):
        raise InvalidLock("duplicate root")
    result = {}
    sources = {}
    for package in packages:
        if not isinstance(package, dict):
            raise InvalidLock("package must be a JSON object")
        name = required_string(package.get("name"), "package name")
        version = required_string(package.get("version"), "package version")
        sha = required_string(package.get("sha256"), "package SHA-256")
        if not SHA256.fullmatch(sha):
            raise InvalidLock("invalid package SHA-256")
        source = required_string(package.get("source"), "package source")
        deps = package.get("dependencies", {})
        if not isinstance(deps, dict) or len(deps) > MAX_PACKAGES:
            raise InvalidLock("invalid package dependencies")
        dependencies = {
            required_string(k, "dependency name"): required_string(v, "dependency version")
            for k, v in deps.items()
        }
        key = (name, version)
        if key in result:
            raise InvalidLock("duplicate package identity")
        result[key] = {"sha256": sha, "dependencies": dict(sorted(dependencies.items()))}
        # Keep source differences observable without leaking local mirror paths.
        sources[key] = hashlib.sha256(source.encode("utf-8")).hexdigest()
    return {"schema": lock["schema"], "roots": sorted(roots), "packages": result}, sources


def compare(left, right):
    a, a_sources = normalize(left)
    b, b_sources = normalize(right)
    differences = []

    def record(field, identity, a_value, b_value):
        differences.append({
            "field": field,
            "identity": identity,
            "left": a_value,
            "right": b_value,
        })

    if a["schema"] != b["schema"]:
        record("schema", None, a["schema"], b["schema"])
    if a["roots"] != b["roots"]:
        record("roots", None, a["roots"], b["roots"])
    for key in sorted(set(a["packages"]) | set(b["packages"])):
        first = a["packages"].get(key)
        second = b["packages"].get(key)
        identity = f"{key[0]}@{key[1]}"
        if first is None or second is None:
            record("package_presence", identity, first is not None, second is not None)
            continue
        for field in ("sha256", "dependencies"):
            if first[field] != second[field]:
                record(field, identity, first[field], second[field])
    source_differences = [
        f"{key[0]}@{key[1]}"
        for key in sorted(set(a_sources) & set(b_sources))
        if a_sources[key] != b_sources[key]
    ]
    return {
        "diagnostic_schema": "commandf.cf11-lock-identity-diagnostic/v1",
        "semantic_identical": not differences,
        "left_package_count": len(a["packages"]),
        "right_package_count": len(b["packages"]),
        "source_difference_identities": source_differences[:MAX_DIFFERENCES],
        "source_difference_count": len(source_differences),
        "differences": differences[:MAX_DIFFERENCES],
        "difference_count": len(differences),
        "truncated_difference_count": max(0, len(differences) - MAX_DIFFERENCES),
        "note": "source is excluded only from semantic identity, as in cf11-multi-version-proof; this is not publisher authentication",
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("first", type=Path, help="first verified commandf.lock")
    parser.add_argument("second", type=Path, help="second independently verified commandf.lock")
    args = parser.parse_args(argv)
    try:
        report = compare(read_lock(args.first), read_lock(args.second))
    except InvalidLock as error:
        print(json.dumps({"diagnostic_schema": "commandf.cf11-lock-identity-diagnostic/v1",
                          "error": str(error), "semantic_identical": False}, sort_keys=True))
        return 2
    print(json.dumps(report, sort_keys=True, indent=2))
    return 0 if report["semantic_identical"] else 1


if __name__ == "__main__":
    sys.exit(main())
