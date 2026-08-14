#!/usr/bin/env python3
"""Build deterministic changed-profile CF-06 CI fixtures from a verified R4 core package."""

from __future__ import annotations

import argparse
import copy
import gzip
import hashlib
import io
import json
import shutil
import tarfile
from pathlib import Path

PACKAGE_NAME = "dev.commandf.oracle.fixture"
CANONICAL = "http://example.org/fhir/StructureDefinition/commandf-oracle-fixture"
SOURCE_CANONICAL = "http://hl7.org/fhir/StructureDefinition/vitalsigns"
CORE_NAME = "hl7.fhir.r4.core"
CORE_VERSION = "4.0.1"
RESOURCE_FILENAME = "StructureDefinition-commandf-oracle-fixture.json"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--core-archive", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def compact_json(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode() + b"\n"


def pretty_json(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False).encode() + b"\n"


def load_source_profile(core_archive: Path) -> dict:
    with tarfile.open(core_archive, "r:gz") as archive:
        for member in archive.getmembers():
            if not member.isfile() or not member.name.startswith("package/") or not member.name.endswith(".json"):
                continue
            handle = archive.extractfile(member)
            if handle is None:
                continue
            try:
                value = json.load(handle)
            except (UnicodeDecodeError, json.JSONDecodeError):
                continue
            if value.get("resourceType") == "StructureDefinition" and value.get("url") == SOURCE_CANONICAL:
                return value
    raise SystemExit(f"unable to find {SOURCE_CANONICAL} in {core_archive}")


def prepare_profile(source: dict, version: str) -> dict:
    profile = copy.deepcopy(source)
    profile["id"] = "commandf-oracle-fixture"
    profile["url"] = CANONICAL
    profile["version"] = version
    profile["name"] = "CommandFOracleFixture"
    profile["title"] = "commandF Oracle Fixture"
    return profile


def mutate_after(profile: dict) -> dict[str, str]:
    snapshot = profile.get("snapshot", {}).get("element")
    if not isinstance(snapshot, list) or not snapshot:
        raise SystemExit("source profile has no usable snapshot")

    evidence: dict[str, str] = {}

    for element in snapshot[1:]:
        minimum = element.get("min")
        maximum = element.get("max")
        if minimum == 0 and maximum not in (None, "0"):
            element["min"] = 1
            evidence["cardinality"] = element.get("id", "")
            break
    if "cardinality" not in evidence:
        raise SystemExit("unable to find snapshot element for cardinality mutation")

    for element in snapshot[1:]:
        types = element.get("type")
        if isinstance(types, list) and len(types) >= 2:
            element["type"] = [copy.deepcopy(types[0])]
            evidence["type"] = element.get("id", "")
            break
    if "type" not in evidence:
        raise SystemExit("unable to find snapshot element for type mutation")

    strength_order = ["example", "preferred", "extensible", "required"]
    for element in snapshot[1:]:
        binding = element.get("binding")
        if not isinstance(binding, dict):
            continue
        strength = binding.get("strength")
        if strength not in strength_order:
            continue
        replacement = "required" if strength != "required" else "extensible"
        binding["strength"] = replacement
        evidence["binding"] = element.get("id", "")
        break
    if "binding" not in evidence:
        raise SystemExit("unable to find snapshot element for binding mutation")

    for element in snapshot[1:]:
        if element.get("mustSupport") is not True:
            element["mustSupport"] = True
            evidence["mustSupport"] = element.get("id", "")
            break
    if "mustSupport" not in evidence:
        raise SystemExit("unable to find snapshot element for mustSupport mutation")

    return evidence


def package_manifest(version: str) -> dict:
    return {
        "name": PACKAGE_NAME,
        "version": version,
        "type": "fhir.ig",
        "fhirVersions": [CORE_VERSION],
        "dependencies": {CORE_NAME: CORE_VERSION},
    }


def package_index(profile: dict) -> dict:
    return {
        "index-version": 2,
        "files": [
            {
                "filename": RESOURCE_FILENAME,
                "resourceType": "StructureDefinition",
                "id": profile["id"],
                "url": profile["url"],
                "version": profile["version"],
                "kind": profile.get("kind"),
                "type": profile.get("type"),
                "derivation": profile.get("derivation"),
            }
        ],
    }


def add_bytes(archive: tarfile.TarFile, name: str, data: bytes) -> None:
    info = tarfile.TarInfo(name)
    info.size = len(data)
    info.mtime = 0
    info.mode = 0o644
    info.uid = 0
    info.gid = 0
    info.uname = ""
    info.gname = ""
    archive.addfile(info, io.BytesIO(data))


def build_package(path: Path, version: str, profile: dict) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w", format=tarfile.PAX_FORMAT) as archive:
                add_bytes(archive, "package/package.json", pretty_json(package_manifest(version)))
                add_bytes(archive, "package/.index.json", pretty_json(package_index(profile)))
                add_bytes(archive, f"package/{RESOURCE_FILENAME}", compact_json(profile))
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_state(
    root: Path,
    core_archive: Path,
    core_digest: str,
    fixture_archive: Path,
    fixture_version: str,
    fixture_digest: str,
) -> None:
    cache = root / "cache" / "sha256"
    cache.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(core_archive, cache / f"{core_digest}.tgz")
    shutil.copyfile(fixture_archive, cache / f"{fixture_digest}.tgz")
    lock = {
        "schema": 1,
        "roots": [f"{PACKAGE_NAME}@{fixture_version}"],
        "packages": [
            {
                "name": PACKAGE_NAME,
                "version": fixture_version,
                "sha256": fixture_digest,
                "source": "cf06-ci-fixture",
                "dependencies": {CORE_NAME: CORE_VERSION},
            },
            {
                "name": CORE_NAME,
                "version": CORE_VERSION,
                "sha256": core_digest,
                "source": "cf06-ci-core",
                "dependencies": {},
            },
        ],
    }
    (root / "commandf.lock").write_bytes(pretty_json(lock))


def main() -> None:
    args = parse_args()
    core_archive = args.core_archive.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)

    source = load_source_profile(core_archive)
    before = prepare_profile(source, "1.0.0")
    after = prepare_profile(source, "1.1.0")
    mutations = mutate_after(after)

    before_archive = output / "before.tgz"
    after_archive = output / "after.tgz"
    before_digest = build_package(before_archive, "1.0.0", before)
    after_digest = build_package(after_archive, "1.1.0", after)
    core_digest = sha256(core_archive)

    write_state(output / "before", core_archive, core_digest, before_archive, "1.0.0", before_digest)
    write_state(output / "after", core_archive, core_digest, after_archive, "1.1.0", after_digest)

    metadata = {
        "package_name": PACKAGE_NAME,
        "canonical": CANONICAL,
        "before_archive": str(before_archive),
        "after_archive": str(after_archive),
        "core_archive": str(core_archive),
        "mutations": mutations,
    }
    (output / "fixture.json").write_bytes(pretty_json(metadata))


if __name__ == "__main__":
    main()
