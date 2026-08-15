#!/usr/bin/env python3
"""Deterministic CF-10 qualification for pinned HL7 comparison failures.

Acquisition is intentionally separate from qualification. The ``acquire`` command uses commandF's
own resolver and cache verifier. The ``qualify`` command consumes only those verified local bytes,
so callers can remove network access for the comparison matrix.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import signal
import subprocess
import tarfile
import threading
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, BinaryIO, Iterable


SCHEMA = 1
ORACLE = {
    "project": "hapifhir/org.hl7.fhir.core",
    "release": "6.10.2",
    "source_commit": "d06577dbc5c62c74a2a8823fbc4830a3024d5b0b",
}
CORE_NAME = "hl7.fhir.r4.core"
CORE_VERSION = "4.0.1"
MAX_LOCK_BYTES = 4 * 1024 * 1024
MAX_JSON_MEMBER_BYTES = 64 * 1024 * 1024
MAX_JSON_MEMBERS = 100_000
MAX_STDOUT_BYTES = 8 * 1024 * 1024
MAX_STDERR_BYTES = 1024 * 1024
MAX_DIAGNOSTIC_CHARS = 8192
PROCESS_TIMEOUT_SECONDS = 180

ROOT_CAUSE_CONTEXT = "CF10_ORACLE_ROOT_CAUSE_COMMAND_F_CONTEXT_DEFECT"
ROOT_CAUSE_PINNED = "CF10_ORACLE_ROOT_CAUSE_PINNED_HL7_COMPARATOR_LIMITATION"
ROOT_CAUSE_MIXED = "CF10_ORACLE_ROOT_CAUSE_MIXED"
ROOT_CAUSE_NOT_PROVEN = "CF10_ORACLE_ROOT_CAUSE_NOT_PROVEN"


@dataclass(frozen=True)
class StateSpec:
    case_id: str
    side: str
    package: str
    version: str
    archive_sha256: str
    archive_bytes: int
    lock_sha256: str


STATE_SPECS = (
    StateSpec(
        "C001",
        "before",
        "hl7.fhir.us.core",
        "8.0.1",
        "3c02eef48ef10617021bee95e58cbc66d596ceda8cada24b72000d33ad67c464",
        2_713_046,
        "2ba6240dc7ffc3c63d1fdaa6597775f083ae1937aa5ec21d34126487632f45ee",
    ),
    StateSpec(
        "C001",
        "after",
        "hl7.fhir.us.core",
        "9.0.0",
        "d7b54d2ec2a48cea94ffea5d939ad67a681f80b94d69594a08cebac36da9e059",
        2_749_959,
        "a88d94cce6743624829bbac5a64464a6d276620961cfb1f072d5518c71236558",
    ),
    StateSpec(
        "C002",
        "before",
        "hl7.fhir.uv.ips",
        "1.1.0",
        "403c4141101810e924f2928287985084819d8a5cc3a62e2b3840a557129840ef",
        1_065_103,
        "1bd1ca2c3c690f9de59c8403a0ea505832a5beecea46d789254a1b7a11f4b3fd",
    ),
    StateSpec(
        "C002",
        "after",
        "hl7.fhir.uv.ips",
        "2.0.1",
        "7183242b70fb2a9058aa3701fb607517a3c2fd0e3100d1d8c538d744c2adf799",
        725_312,
        "b2485f1caa7872aa3d95bcc3254addd9204fe62a59769265de7f271ed14b3953",
    ),
)

CASE_HYPOTHESES = {
    "C001": {
        "canonical_url": (
            "http://hl7.org/fhir/us/core/StructureDefinition/"
            "head-occipital-frontal-circumference-percentile"
        ),
        "failing_path": "Observation.category",
    },
    "C002": {
        "canonical_url": "http://hl7.org/fhir/uv/ips/StructureDefinition/Composition-uv-ips",
        "failing_path": "Composition.section",
    },
}


@dataclass(frozen=True)
class VerifiedState:
    spec: StateSpec
    root: Path
    lock: dict[str, Any]
    root_package: dict[str, Any]
    core_package: dict[str, Any]

    def archive(self, package: dict[str, Any]) -> Path:
        return self.root / "cache" / "sha256" / f"{package['sha256']}.tgz"


@dataclass(frozen=True)
class ProfileResource:
    filename: str
    value: dict[str, Any]
    raw_sha256: str | None = None


@dataclass(frozen=True)
class MatchedProfilePair:
    resource_key: str
    canonical_url: str
    lookup_version: str | None
    before: ProfileResource
    after: ProfileResource


@dataclass(frozen=True)
class ProcessResult:
    returncode: int
    stdout: bytes
    stderr: bytes


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def bounded_text(value: str, limit: int = MAX_DIAGNOSTIC_CHARS) -> str:
    if len(value) <= limit:
        return value
    return value[:limit] + "... [diagnostic truncated]"


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode(
        "utf-8"
    )


def read_bounded(path: Path, limit: int) -> bytes:
    with path.open("rb") as stream:
        value = stream.read(limit + 1)
    if len(value) > limit:
        raise ValueError(f"{path.name} exceeds {limit} bytes")
    return value


def _drain(stream: BinaryIO, limit: int, retained: bytearray, total: list[int]) -> None:
    while True:
        chunk = stream.read(64 * 1024)
        if not chunk:
            return
        total[0] += len(chunk)
        remaining = limit + 1 - len(retained)
        if remaining > 0:
            retained.extend(chunk[:remaining])


def run_bounded(argv: list[str], timeout: int = PROCESS_TIMEOUT_SECONDS) -> ProcessResult:
    process = subprocess.Popen(
        argv,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    assert process.stdout is not None
    assert process.stderr is not None
    stdout = bytearray()
    stderr = bytearray()
    stdout_total = [0]
    stderr_total = [0]
    threads = [
        threading.Thread(
            target=_drain,
            args=(process.stdout, MAX_STDOUT_BYTES, stdout, stdout_total),
            daemon=True,
        ),
        threading.Thread(
            target=_drain,
            args=(process.stderr, MAX_STDERR_BYTES, stderr, stderr_total),
            daemon=True,
        ),
    ]
    for thread in threads:
        thread.start()
    try:
        returncode = process.wait(timeout=timeout)
    except subprocess.TimeoutExpired as error:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except (AttributeError, OSError):
            process.kill()
        process.wait()
        for thread in threads:
            thread.join()
        raise RuntimeError(f"process timed out after {timeout} seconds") from error
    for thread in threads:
        thread.join()
    if stdout_total[0] > MAX_STDOUT_BYTES:
        raise RuntimeError(
            f"process stdout exceeded {MAX_STDOUT_BYTES} bytes: {stdout_total[0]}"
        )
    if stderr_total[0] > MAX_STDERR_BYTES:
        raise RuntimeError(
            f"process stderr exceeded {MAX_STDERR_BYTES} bytes: {stderr_total[0]}"
        )
    return ProcessResult(returncode, bytes(stdout), bytes(stderr))


def state_path(work_root: Path, spec: StateSpec) -> Path:
    return work_root / "states" / spec.case_id / spec.side


def identity(package: dict[str, Any]) -> dict[str, str]:
    return {
        "name": required_string(package, "name"),
        "version": required_string(package, "version"),
        "sha256": required_digest(package, "sha256"),
    }


def required_string(value: dict[str, Any], key: str) -> str:
    result = value.get(key)
    if not isinstance(result, str) or not result:
        raise ValueError(f"{key} must be a non-empty string")
    return result


def required_digest(value: dict[str, Any], key: str) -> str:
    result = required_string(value, key)
    if len(result) != 64 or result != result.lower() or any(
        char not in "0123456789abcdef" for char in result
    ):
        raise ValueError(f"{key} must be lowercase SHA-256")
    return result


def package_identity(package: dict[str, Any]) -> tuple[str, str]:
    return required_string(package, "name"), required_string(package, "version")


def select_exact(
    lock: dict[str, Any], name: str, version: str, role: str
) -> dict[str, Any]:
    packages = lock.get("packages")
    if not isinstance(packages, list):
        raise ValueError("lock packages must be an array")
    matches = [
        package
        for package in packages
        if isinstance(package, dict)
        and package.get("name") == name
        and package.get("version") == version
    ]
    if len(matches) != 1:
        raise ValueError(f"{role} {name}@{version} matched {len(matches)} locked packages")
    return matches[0]


def select_dependency(
    lock: dict[str, Any], parent: dict[str, Any], name: str, constraint: str
) -> dict[str, Any]:
    # The retained CF-10 locks use exact dependency constraints. Refuse to invent semver behavior
    # in this diagnostic rather than accidentally selecting a different concrete identity.
    if not isinstance(constraint, str) or not constraint:
        raise ValueError("dependency constraint must be a non-empty string")
    return select_exact(
        lock,
        name,
        constraint,
        f"dependency of {parent['name']}@{parent['version']}",
    )


def verify_state(work_root: Path, spec: StateSpec) -> VerifiedState:
    root = state_path(work_root, spec)
    lock_path = root / "commandf.lock"
    lock_bytes = read_bounded(lock_path, MAX_LOCK_BYTES)
    actual_lock_sha = sha256_bytes(lock_bytes)
    if actual_lock_sha != spec.lock_sha256:
        raise ValueError(
            f"{spec.case_id} {spec.side} lock digest mismatch: "
            f"{actual_lock_sha} != {spec.lock_sha256}"
        )
    lock = json.loads(lock_bytes)
    if lock.get("schema") != 1:
        raise ValueError("lock schema must be 1")
    if lock.get("roots") != [f"{spec.package}@{spec.version}"]:
        raise ValueError(f"{spec.case_id} {spec.side} root request mismatch")
    root_package = select_exact(lock, spec.package, spec.version, "root")
    core_package = select_exact(lock, CORE_NAME, CORE_VERSION, "core")
    if required_digest(root_package, "sha256") != spec.archive_sha256:
        raise ValueError(f"{spec.case_id} {spec.side} root archive digest mismatch")

    packages = lock.get("packages")
    assert isinstance(packages, list)
    seen: dict[tuple[str, str], str] = {}
    for package in packages:
        if not isinstance(package, dict):
            raise ValueError("locked package must be an object")
        identity_key = package_identity(package)
        package_digest = required_digest(package, "sha256")
        if identity_key in seen:
            raise ValueError(
                f"duplicate locked package identity {identity_key[0]}@{identity_key[1]}: "
                f"{seen[identity_key]} and {package_digest}"
            )
        seen[identity_key] = package_digest
        archive = root / "cache" / "sha256" / f"{package_digest}.tgz"
        if not archive.is_file():
            raise ValueError(
                f"missing verified archive for {identity_key[0]}@{identity_key[1]}"
            )
        actual_digest = sha256_file(archive)
        if actual_digest != package_digest:
            raise ValueError(
                f"archive digest mismatch for {identity_key[0]}@{identity_key[1]}"
            )
    root_archive = root / "cache" / "sha256" / f"{spec.archive_sha256}.tgz"
    if root_archive.stat().st_size != spec.archive_bytes:
        raise ValueError(f"{spec.case_id} {spec.side} root archive size mismatch")
    return VerifiedState(spec, root, lock, root_package, core_package)


def is_fhir_core(name: str) -> bool:
    return name.startswith("hl7.fhir.r") and name.endswith(".core")


def root_core_family(state: VerifiedState) -> str:
    dependencies = state.root_package.get("dependencies")
    if not isinstance(dependencies, dict):
        raise ValueError("root dependencies must be an object")
    cores = sorted(name for name in dependencies if is_fhir_core(name))
    if cores != [CORE_NAME]:
        raise ValueError(f"root must declare exactly {CORE_NAME}: {cores}")
    return cores[0]


def package_core_families(
    lock: dict[str, Any], package: dict[str, Any]
) -> set[str]:
    memo: dict[tuple[str, str], frozenset[str]] = {}
    visiting: set[tuple[str, str]] = set()

    def visit(current: dict[str, Any]) -> frozenset[str]:
        key = package_identity(current)
        if key in memo:
            return memo[key]
        if key in visiting:
            raise ValueError(
                f"dependency cycle while scoping core family at {key[0]}@{key[1]}"
            )
        if is_fhir_core(key[0]):
            result = frozenset((key[0],))
            memo[key] = result
            return result
        dependencies = current.get("dependencies")
        if not isinstance(dependencies, dict):
            raise ValueError(f"dependencies for {key[0]}@{key[1]} must be an object")
        visiting.add(key)
        try:
            families: set[str] = set()
            for dependency_name in sorted(dependencies):
                dependency = select_dependency(
                    lock, current, dependency_name, dependencies[dependency_name]
                )
                families.update(visit(dependency))
            result = frozenset(families)
            memo[key] = result
            return result
        finally:
            visiting.remove(key)

    return set(visit(package))


def direct_context_packages(state: VerifiedState) -> list[dict[str, Any]]:
    dependencies = state.root_package.get("dependencies")
    if not isinstance(dependencies, dict):
        raise ValueError("root dependencies must be an object")
    output = []
    for name in sorted(dependencies):
        package = select_dependency(
            state.lock, state.root_package, name, dependencies[name]
        )
        if package_identity(package) == (CORE_NAME, CORE_VERSION):
            continue
        output.append(package)
    return output


def full_context_packages(state: VerifiedState) -> list[dict[str, Any]]:
    target_core = root_core_family(state)
    selected: dict[tuple[str, str], dict[str, Any]] = {}
    traversed: set[tuple[str, str]] = set()

    def include_from_r4_branch(package: dict[str, Any]) -> None:
        key = package_identity(package)
        if key in traversed:
            return
        traversed.add(key)
        if key == (CORE_NAME, CORE_VERSION):
            return
        if is_fhir_core(key[0]):
            return
        dependencies = package.get("dependencies")
        if not isinstance(dependencies, dict):
            raise ValueError(f"dependencies for {key[0]}@{key[1]} must be an object")
        direct_cores = {name for name in dependencies if is_fhir_core(name)}
        if target_core in direct_cores:
            other_cores = direct_cores - {target_core}
            if other_cores:
                raise ValueError(
                    f"mixed direct FHIR core families at {key[0]}@{key[1]}: "
                    f"{sorted(direct_cores)}"
                )
        else:
            families = package_core_families(state.lock, package)
            if target_core in families and len(families) > 1:
                raise ValueError(
                    f"mixed transitive FHIR core families at {key[0]}@{key[1]}: "
                    f"{sorted(families)}"
                )
            if families and families != {target_core}:
                return
        if key in selected:
            raise ValueError(f"duplicate locked package identity {key[0]}@{key[1]}")
        selected[key] = package
        for dependency_name in sorted(dependencies):
            dependency = select_dependency(
                state.lock, package, dependency_name, dependencies[dependency_name]
            )
            include_from_r4_branch(dependency)

    root_dependencies = state.root_package.get("dependencies")
    if not isinstance(root_dependencies, dict):
        raise ValueError("root dependencies must be an object")
    for dependency_name in sorted(root_dependencies):
        dependency = select_dependency(
            state.lock,
            state.root_package,
            dependency_name,
            root_dependencies[dependency_name],
        )
        include_from_r4_branch(dependency)

    ordered: list[dict[str, Any]] = []
    visiting: set[tuple[str, str]] = set()
    visited: set[tuple[str, str]] = set()

    def visit(package: dict[str, Any]) -> None:
        key = package_identity(package)
        if key in visited:
            return
        if key in visiting:
            raise ValueError(f"dependency cycle while ordering context at {key[0]}@{key[1]}")
        visiting.add(key)
        dependencies = package.get("dependencies")
        if not isinstance(dependencies, dict):
            raise ValueError(f"dependencies for {key[0]}@{key[1]} must be an object")
        for dependency_name in sorted(dependencies):
            dependency = select_dependency(
                state.lock, package, dependency_name, dependencies[dependency_name]
            )
            if package_identity(dependency) in selected:
                visit(dependency)
        visiting.remove(key)
        visited.add(key)
        ordered.append(package)

    for key in sorted(selected):
        visit(selected[key])
    return ordered


def context_packages(state: VerifiedState, mode: str) -> list[dict[str, Any]]:
    if mode == "DIRECT":
        return direct_context_packages(state)
    if mode == "FULL_CLOSURE":
        return full_context_packages(state)
    raise ValueError(f"unknown context mode {mode}")


def iter_json_resources(
    archive: Path,
) -> Iterable[tuple[str, dict[str, Any], str]]:
    count = 0
    with tarfile.open(archive, mode="r:gz") as package:
        for member in package:
            if not member.isfile() or not member.name.startswith("package/"):
                continue
            if not member.name.endswith(".json") or member.name == "package/package.json":
                continue
            count += 1
            if count > MAX_JSON_MEMBERS:
                raise ValueError(f"archive contains more than {MAX_JSON_MEMBERS} JSON resources")
            if member.size > MAX_JSON_MEMBER_BYTES:
                raise ValueError(f"JSON member {member.name} exceeds size limit")
            stream = package.extractfile(member)
            if stream is None:
                raise ValueError(f"unable to read {member.name}")
            raw = stream.read(MAX_JSON_MEMBER_BYTES + 1)
            if len(raw) > MAX_JSON_MEMBER_BYTES:
                raise ValueError(f"JSON member {member.name} exceeds size limit")
            value = json.loads(raw)
            if not isinstance(value, dict):
                raise ValueError(f"FHIR resource {member.name} must be an object")
            yield PurePosixPath(member.name).name, value, sha256_bytes(raw)


def find_profile(
    state: VerifiedState, canonical_url: str, lookup_version: str | None = None
) -> ProfileResource:
    matches = []
    for filename, value, raw_sha256 in iter_json_resources(
        state.archive(state.root_package)
    ):
        if (
            value.get("resourceType") == "StructureDefinition"
            and value.get("url") == canonical_url
            and (lookup_version is None or value.get("version") == lookup_version)
        ):
            matches.append(ProfileResource(filename, value, raw_sha256))
    if len(matches) != 1:
        identity_text = canonical_url
        if lookup_version is not None:
            identity_text += f"|{lookup_version}"
        raise ValueError(
            f"{state.spec.case_id} {state.spec.side} canonical "
            f"{identity_text} matched {len(matches)} profiles"
        )
    return matches[0]


def root_canonical_inventory(
    state: VerifiedState,
) -> tuple[dict[str, int], list[ProfileResource]]:
    counts: dict[str, int] = {}
    profiles = []
    for filename, value, raw_sha256 in iter_json_resources(
        state.archive(state.root_package)
    ):
        url = value.get("url")
        if isinstance(url, str) and url:
            counts[url] = counts.get(url, 0) + 1
        if value.get("resourceType") == "StructureDefinition" and isinstance(url, str) and url:
            profiles.append(ProfileResource(filename, value, raw_sha256))
    return counts, profiles


def profile_resource_key(
    profile: ProfileResource,
    before_counts: dict[str, int],
    after_counts: dict[str, int],
) -> tuple[str, str, str | None]:
    url = profile.value.get("url")
    if not isinstance(url, str) or not url:
        raise ValueError(f"{profile.filename} has no canonical URL")
    if before_counts.get(url, 0) <= 1 and after_counts.get(url, 0) <= 1:
        return url, url, None
    version = profile.value.get("version")
    if not isinstance(version, str) or not version.strip():
        raise ValueError(
            f"canonical multiplicity requires a usable version in {profile.filename}"
        )
    return f"{url}|{version}", url, version


def structural_changed_canonical_keys(report: dict[str, Any]) -> set[str]:
    if report.get("schema") != 1:
        raise ValueError("structural report schema must be 1")
    changes = report.get("changes")
    if not isinstance(changes, list):
        raise ValueError("structural report changes must be an array")
    keys: set[str] = set()
    for change in changes:
        if not isinstance(change, dict):
            raise ValueError("structural change must be an object")
        resource = change.get("resource")
        if not isinstance(resource, dict):
            raise ValueError("structural change resource must be an object")
        if resource.get("kind") != "canonical":
            continue
        value = resource.get("value")
        if not isinstance(value, str) or not value:
            raise ValueError("canonical structural resource key must be non-empty")
        keys.add(value)
    return keys


def matched_changed_profiles(
    before: VerifiedState,
    after: VerifiedState,
    structural_report: dict[str, Any],
) -> list[MatchedProfilePair]:
    before_counts, before_profiles = root_canonical_inventory(before)
    after_counts, after_profiles = root_canonical_inventory(after)

    def build_index(profiles: list[ProfileResource]) -> dict[str, tuple[str, str | None, ProfileResource]]:
        index: dict[str, tuple[str, str | None, ProfileResource]] = {}
        for profile in profiles:
            key, url, version = profile_resource_key(
                profile, before_counts, after_counts
            )
            if key in index:
                raise ValueError(f"duplicate StructureDefinition resource key {key}")
            index[key] = (url, version, profile)
        return index

    before_index = build_index(before_profiles)
    after_index = build_index(after_profiles)
    changed = structural_changed_canonical_keys(structural_report)
    output = []
    for key in sorted(before_index.keys() & after_index.keys() & changed):
        before_url, before_version, before_profile = before_index[key]
        after_url, after_version, after_profile = after_index[key]
        if (before_url, before_version) != (after_url, after_version):
            raise ValueError(f"matched StructureDefinition identity differs for {key}")
        output.append(
            MatchedProfilePair(
                key,
                before_url,
                before_version,
                before_profile,
                after_profile,
            )
        )
    return output


def element_view_shape(value: dict[str, Any], view: str, failing_path: str) -> dict[str, Any]:
    container = value.get(view)
    elements = container.get("element", []) if isinstance(container, dict) else []
    if not isinstance(elements, list):
        raise ValueError(f"{view}.element must be an array")

    exact_indices = [
        index
        for index, element in enumerate(elements)
        if isinstance(element, dict) and element.get("path") == failing_path
    ]
    window_indices: set[int] = set()
    for index in exact_indices:
        window_indices.update(
            candidate
            for candidate in (index - 1, index, index + 1)
            if 0 <= candidate < len(elements)
        )

    def element_evidence(index: int) -> dict[str, Any]:
        element = elements[index]
        assert isinstance(element, dict)
        return {
            "index": index,
            "path": element.get("path"),
            "id": element.get("id"),
            "slice_name": element.get("sliceName"),
            "has_slicing": "slicing" in element,
            "slicing": element.get("slicing"),
            "raw_element": element,
        }

    exact = [element_evidence(index) for index in exact_indices]
    named_indices = [
        item["index"] for item in exact if isinstance(item.get("slice_name"), str)
    ]
    slicing_indices = [item["index"] for item in exact if item["has_slicing"]]
    first_named = min(named_indices) if named_indices else None
    local_slicing_before_named = (
        first_named is not None
        and any(index < first_named for index in slicing_indices)
    )
    return {
        "exists": isinstance(container, dict),
        "element_count": len(elements),
        "path_elements": exact,
        "window": [element_evidence(index) for index in sorted(window_indices)],
        "first_named_slice_index": first_named,
        "slicing_indices": slicing_indices,
        "slicing_before_first_named_slice": local_slicing_before_named,
        "named_slice_precedes_slicing": first_named is not None
        and not local_slicing_before_named,
    }


def differential_shape(profile: ProfileResource, failing_path: str) -> dict[str, Any]:
    value = profile.value
    return {
        "url": value.get("url"),
        "version": value.get("version"),
        "resource_sha256": profile.raw_sha256,
        "base_definition": value.get("baseDefinition"),
        "derivation": value.get("derivation"),
        "snapshot_exists": isinstance(value.get("snapshot"), dict),
        "differential_exists": isinstance(value.get("differential"), dict),
        "failing_path": failing_path,
        "differential": element_view_shape(value, "differential", failing_path),
        "snapshot": element_view_shape(value, "snapshot", failing_path),
    }


def context_evidence(
    state: VerifiedState, packages: list[dict[str, Any]]
) -> dict[str, Any]:
    core = identity(state.core_package)
    subject = identity(state.root_package)
    additional = [identity(package) for package in packages]
    return {
        "core": core,
        "additional_packages": additional,
        "subject": subject,
        "load_order": [core, *additional, subject],
    }


def profile_identity(profile: ProfileResource) -> dict[str, str | None]:
    def optional_string(key: str) -> str | None:
        value = profile.value.get(key)
        return value if isinstance(value, str) and value.strip() else None

    return {
        "url": optional_string("url"),
        "version": optional_string("version"),
        "id": optional_string("id"),
        "type": optional_string("type"),
    }


def probe_argv(
    java: Path,
    probe_classes: Path,
    oracle_jar: Path,
    pair: MatchedProfilePair,
    left: VerifiedState,
    right: VerifiedState,
    left_profile: ProfileResource,
    right_profile: ProfileResource,
    left_context: list[dict[str, Any]],
    right_context: list[dict[str, Any]],
) -> list[str]:
    if required_digest(left.core_package, "sha256") != required_digest(
        right.core_package, "sha256"
    ):
        raise ValueError("left/right R4 core package digests differ")
    argv = [
        str(java),
        "-cp",
        os.pathsep.join((str(probe_classes), str(oracle_jar))),
        "dev.commandf.oracle.qualification.QualificationProbe",
        "--core-package",
        str(left.archive(left.core_package)),
        "--left-package",
        str(left.archive(left.root_package)),
        "--right-package",
        str(right.archive(right.root_package)),
    ]
    for package in left_context:
        argv.extend(["--left-context-package", str(left.archive(package))])
    for package in right_context:
        argv.extend(["--right-context-package", str(right.archive(package))])
    argv.extend(["--left-url", pair.canonical_url])
    if pair.lookup_version is not None:
        argv.extend(["--left-version", pair.lookup_version])
    argv.extend(["--right-url", pair.canonical_url])
    if pair.lookup_version is not None:
        argv.extend(["--right-version", pair.lookup_version])
    return argv


def invocation_evidence(
    label: str,
    mode: str,
    pair: MatchedProfilePair,
    left: VerifiedState,
    right: VerifiedState,
    left_profile: ProfileResource,
    right_profile: ProfileResource,
    left_context: list[dict[str, Any]],
    right_context: list[dict[str, Any]],
) -> dict[str, Any]:
    return {
        "case_id": left.spec.case_id,
        "probe": label,
        "context_mode": mode,
        "package": left.spec.package,
        "resource_key": pair.resource_key,
        "canonical_url": pair.canonical_url,
        "lookup_version": pair.lookup_version,
        "left": {
            "package": identity(left.root_package),
            "canonical_version": left_profile.value.get("version"),
            "filename": left_profile.filename,
            "resource": profile_identity(left_profile),
            "context": context_evidence(left, left_context),
        },
        "right": {
            "package": identity(right.root_package),
            "canonical_version": right_profile.value.get("version"),
            "filename": right_profile.filename,
            "resource": profile_identity(right_profile),
            "context": context_evidence(right, right_context),
        },
    }


def run_probe(
    work_root: Path,
    invocation: dict[str, Any],
    argv: list[str],
) -> dict[str, Any]:
    event = {"event": "probe_start", **invocation}
    print(json.dumps(event, sort_keys=True, separators=(",", ":")), flush=True)
    process = run_bounded(argv)
    stdout = process.stdout.decode("utf-8")
    stderr = process.stderr.decode("utf-8")
    forbidden_paths = {str(work_root.resolve())}
    for index, argument in enumerate(argv):
        candidates = (
            argument.split(os.pathsep)
            if index > 0 and argv[index - 1] in {"-cp", "--class-path"}
            else (argument,)
        )
        for value in candidates:
            candidate = Path(value)
            if candidate.is_absolute():
                forbidden_paths.add(str(candidate))
    if any(path in stdout or path in stderr for path in forbidden_paths):
        raise ValueError("probe output contains a host-absolute path")
    result = json.loads(stdout)
    if result.get("schema") != 1 or result.get("oracle") != ORACLE:
        raise ValueError("qualification probe schema/oracle identity mismatch")
    status = result.get("status")
    if status in {"agreement", "uncomparable"}:
        raise ValueError("qualification exception was coerced into an oracle evidence status")
    if status not in {"completed", "exception"}:
        raise ValueError(f"unexpected qualification status {status}")
    if status == "completed" and process.returncode != 0:
        raise ValueError("completed comparison exited non-zero")
    if status == "exception" and process.returncode == 0:
        raise ValueError("comparison exception exited zero")
    if result.get("phase") == "comparison":
        if result.get("left_resource") != invocation["left"]["resource"]:
            raise ValueError("resolved left StructureDefinition identity mismatch")
        if result.get("right_resource") != invocation["right"]["resource"]:
            raise ValueError("resolved right StructureDefinition identity mismatch")
    return {
        **invocation,
        "process_exit_code": process.returncode,
        "process_stderr": bounded_text(stderr.replace("\r\n", "\n").rstrip()),
        "result": result,
    }


def is_slice_definition_failure(result: dict[str, Any]) -> bool:
    return (
        result.get("status") == "exception"
        and result.get("phase") == "comparison"
        and result.get("exception_class")
        == "org.hl7.fhir.exceptions.DefinitionException"
        and exception_path(result) is not None
    )


def failure_signature(result: dict[str, Any]) -> tuple[Any, ...]:
    return (
        result.get("status"),
        result.get("phase"),
        result.get("exception_class"),
        result.get("exception_message"),
        exception_path(result),
    )


def case_classification(
    case_id: str,
    probes: list[dict[str, Any]],
    discovered_result: dict[str, Any] | None,
) -> str:
    if discovered_result is None or not is_slice_definition_failure(discovered_result):
        return ROOT_CAUSE_NOT_PROVEN
    case_probes = [probe for probe in probes if probe["case_id"] == case_id]
    by_key = {
        (probe["probe"], probe["context_mode"]): probe["result"]
        for probe in case_probes
    }
    required = {
        (label, mode)
        for label in ("self_before", "self_after", "cross")
        for mode in ("DIRECT", "FULL_CLOSURE")
    }
    if not required.issubset(by_key):
        return ROOT_CAUSE_NOT_PROVEN
    discovered_signature = failure_signature(discovered_result)
    cross_direct = by_key[("cross", "DIRECT")]
    cross_full = by_key[("cross", "FULL_CLOSURE")]
    if failure_signature(cross_direct) != discovered_signature:
        return ROOT_CAUSE_NOT_PROVEN
    context_fixed = (
        cross_full.get("status") == "completed"
        and cross_full.get("phase") == "comparison"
    )
    pinned_self_failure = False
    for label in ("self_before", "self_after"):
        direct = by_key[(label, "DIRECT")]
        full = by_key[(label, "FULL_CLOSURE")]
        if (
            failure_signature(direct)
            == failure_signature(full)
            == discovered_signature
        ):
            pinned_self_failure = True
    pinned_cross_failure = failure_signature(cross_full) == discovered_signature
    if pinned_self_failure and context_fixed:
        return ROOT_CAUSE_NOT_PROVEN
    if pinned_self_failure and pinned_cross_failure:
        return ROOT_CAUSE_PINNED
    if context_fixed:
        return ROOT_CAUSE_CONTEXT
    return ROOT_CAUSE_NOT_PROVEN


def primary_classification(case_classes: dict[str, str]) -> str:
    values = set(case_classes.values())
    if values == {ROOT_CAUSE_PINNED}:
        return ROOT_CAUSE_PINNED
    if values == {ROOT_CAUSE_CONTEXT}:
        return ROOT_CAUSE_CONTEXT
    if values == {ROOT_CAUSE_PINNED, ROOT_CAUSE_CONTEXT}:
        return ROOT_CAUSE_MIXED
    return ROOT_CAUSE_NOT_PROVEN


def structural_report_path(work_root: Path, case_id: str) -> Path:
    return work_root / "cases" / case_id / "structural.json"


def validate_structural_report(
    report: dict[str, Any], before: VerifiedState, after: VerifiedState
) -> None:
    if report.get("schema") != 1:
        raise ValueError("structural report schema must be 1")
    if report.get("package_name") != before.spec.package:
        raise ValueError(f"{before.spec.case_id} structural package identity mismatch")
    expected_before = {
        "version": before.spec.version,
        "archive_sha256": before.spec.archive_sha256,
    }
    expected_after = {
        "version": after.spec.version,
        "archive_sha256": after.spec.archive_sha256,
    }
    if report.get("before") != expected_before or report.get("after") != expected_after:
        raise ValueError(f"{before.spec.case_id} structural state evidence mismatch")
    structural_changed_canonical_keys(report)


def load_structural_report(
    work_root: Path, before: VerifiedState, after: VerifiedState
) -> dict[str, Any]:
    report = json.loads(
        read_bounded(structural_report_path(work_root, before.spec.case_id), MAX_STDOUT_BYTES)
    )
    if not isinstance(report, dict):
        raise ValueError("structural report must be an object")
    validate_structural_report(report, before, after)
    return report


def acquire(commandf: Path, work_root: Path, output: Path) -> None:
    if work_root.exists():
        raise ValueError(f"qualification work root already exists: {work_root.name}")
    for spec in STATE_SPECS:
        root = state_path(work_root, spec)
        root.mkdir(parents=True, exist_ok=False)
        resolve = run_bounded(
            [
                str(commandf),
                "pkg",
                "resolve",
                f"{spec.package}@{spec.version}",
                "--cache",
                str(root / "cache"),
                "--lock",
                str(root / "commandf.lock"),
            ],
            timeout=600,
        )
        if resolve.returncode != 0:
            raise RuntimeError(
                f"resolver failed for {spec.case_id} {spec.side}: "
                + bounded_text(resolve.stderr.decode("utf-8", errors="replace"))
            )
        verify = run_bounded(
            [
                str(commandf),
                "pkg",
                "verify",
                "--cache",
                str(root / "cache"),
                "--lock",
                str(root / "commandf.lock"),
            ]
        )
        if verify.returncode != 0:
            raise RuntimeError(f"cache verification failed for {spec.case_id} {spec.side}")
        verify_state(work_root, spec)
    states = [verify_state(work_root, spec) for spec in STATE_SPECS]
    states_by_key = {
        (state.spec.case_id, state.spec.side): state for state in states
    }
    structural_evidence = []
    for case_id in ("C001", "C002"):
        before = states_by_key[(case_id, "before")]
        after = states_by_key[(case_id, "after")]
        process = run_bounded(
            [
                str(commandf),
                "diff",
                before.spec.package,
                "--before-lock",
                str(before.root / "commandf.lock"),
                "--before-cache",
                str(before.root / "cache"),
                "--after-lock",
                str(after.root / "commandf.lock"),
                "--after-cache",
                str(after.root / "cache"),
            ]
        )
        if process.returncode != 0:
            raise RuntimeError(
                f"structural diff failed for {case_id}: "
                + bounded_text(process.stderr.decode("utf-8", errors="replace"))
            )
        report = json.loads(process.stdout)
        if not isinstance(report, dict):
            raise ValueError(f"{case_id} structural report must be an object")
        validate_structural_report(report, before, after)
        report_bytes = canonical_json_bytes(report)
        report_path = structural_report_path(work_root, case_id)
        report_path.parent.mkdir(parents=True, exist_ok=False)
        report_path.write_bytes(report_bytes)
        structural_evidence.append(
            {
                "case_id": case_id,
                "sha256": sha256_bytes(report_bytes),
                "changed_canonical_count": len(
                    structural_changed_canonical_keys(report)
                ),
            }
        )
    evidence = {
        "schema": SCHEMA,
        "states": [
            {
                "case_id": state.spec.case_id,
                "side": state.spec.side,
                "root": identity(state.root_package),
                "lock_sha256": state.spec.lock_sha256,
                "closure": [identity(package) for package in state.lock["packages"]],
            }
            for state in states
        ],
        "structural_reports": structural_evidence,
    }
    output.write_bytes(canonical_json_bytes(evidence))


SLICE_FAILURE = re.compile(
    r"Found a slice at '([^']+)'\s+but\s+there was no definition for the slicing"
)


def exception_path(result: dict[str, Any]) -> str | None:
    message = result.get("exception_message")
    if not isinstance(message, str):
        return None
    match = SLICE_FAILURE.search(message.replace("\r", " ").replace("\n", " "))
    return match.group(1) if match else None


def execute_pair_probe(
    java: Path,
    probe_classes: Path,
    oracle_jar: Path,
    work_root: Path,
    pair: MatchedProfilePair,
    label: str,
    mode: str,
    left: VerifiedState,
    right: VerifiedState,
    left_profile: ProfileResource,
    right_profile: ProfileResource,
    extra_evidence: dict[str, Any] | None = None,
) -> dict[str, Any]:
    left_context = context_packages(left, mode)
    right_context = context_packages(right, mode)
    invocation = invocation_evidence(
        label,
        mode,
        pair,
        left,
        right,
        left_profile,
        right_profile,
        left_context,
        right_context,
    )
    if extra_evidence:
        invocation.update(extra_evidence)
    argv = probe_argv(
        java,
        probe_classes,
        oracle_jar,
        pair,
        left,
        right,
        left_profile,
        right_profile,
        left_context,
        right_context,
    )
    return run_probe(work_root, invocation, argv)


def context_mode_comparisons(probes: list[dict[str, Any]]) -> list[dict[str, Any]]:
    output = []
    for case_id in ("C001", "C002"):
        for label in ("self_before", "self_after", "cross"):
            selected = {
                probe["context_mode"]: probe["result"]
                for probe in probes
                if probe["case_id"] == case_id and probe["probe"] == label
            }
            if set(selected) != {"DIRECT", "FULL_CLOSURE"}:
                continue
            direct = selected["DIRECT"]
            full = selected["FULL_CLOSURE"]
            direct_path = exception_path(direct)
            full_path = exception_path(full)
            output.append(
                {
                    "case_id": case_id,
                    "probe": label,
                    "direct_status": direct.get("status"),
                    "full_closure_status": full.get("status"),
                    "exception_class_changed": direct.get("exception_class")
                    != full.get("exception_class"),
                    "exception_path_changed": direct_path != full_path,
                    "direct_exception_path": direct_path,
                    "full_closure_exception_path": full_path,
                }
            )
    return output


def qualify(
    java: Path,
    probe_classes: Path,
    oracle_jar: Path,
    work_root: Path,
    output: Path,
) -> None:
    states = {
        (spec.case_id, spec.side): verify_state(work_root, spec) for spec in STATE_SPECS
    }
    probes = []
    discovery_probes = []
    discoveries = []
    shapes = []
    discovered_results: dict[str, dict[str, Any] | None] = {}

    for case_id in ("C001", "C002"):
        before = states[(case_id, "before")]
        after = states[(case_id, "after")]
        structural = load_structural_report(work_root, before, after)
        candidates = matched_changed_profiles(before, after, structural)
        selected: MatchedProfilePair | None = None
        selected_result: dict[str, Any] | None = None
        selected_index: int | None = None
        terminal_result: dict[str, Any] | None = None
        for index, pair in enumerate(candidates):
            probe = execute_pair_probe(
                java,
                probe_classes,
                oracle_jar,
                work_root,
                pair,
                "discovery_cross",
                "DIRECT",
                before,
                after,
                pair.before,
                pair.after,
                {
                    "candidate_index": index,
                    "candidate_count": len(candidates),
                },
            )
            discovery_probes.append(probe)
            if probe["result"].get("status") == "exception":
                terminal_result = probe["result"]
                if is_slice_definition_failure(terminal_result):
                    selected = pair
                    selected_result = terminal_result
                    selected_index = index
                break

        observed_path = exception_path(selected_result or {})
        discovered_results[case_id] = selected_result
        hypothesis = CASE_HYPOTHESES[case_id]
        discovery = {
            "case_id": case_id,
            "changed_matched_structure_definition_count": len(candidates),
            "selected_candidate_index": selected_index,
            "selected_resource_key": selected.resource_key if selected else None,
            "selected_canonical_url": selected.canonical_url if selected else None,
            "selected_lookup_version": selected.lookup_version if selected else None,
            "observed_exception_path": observed_path,
            "terminal_exception_phase": (
                terminal_result.get("phase") if terminal_result else None
            ),
            "terminal_exception_class": (
                terminal_result.get("exception_class") if terminal_result else None
            ),
            "terminal_exception_message": (
                terminal_result.get("exception_message") if terminal_result else None
            ),
            "hypothesis": hypothesis,
            "hypothesis_canonical_confirmed": selected is not None
            and selected.canonical_url == hypothesis["canonical_url"],
            "hypothesis_path_confirmed": observed_path == hypothesis["failing_path"],
        }
        discoveries.append(discovery)
        if selected is None:
            continue

        if observed_path is not None:
            for side, profile in (("before", selected.before), ("after", selected.after)):
                shapes.append(
                    {
                        "case_id": case_id,
                        "side": side,
                        "filename": profile.filename,
                        **differential_shape(profile, observed_path),
                    }
                )

        combinations = (
            ("self_before", before, before, selected.before, selected.before),
            ("self_after", after, after, selected.after, selected.after),
            ("cross", before, after, selected.before, selected.after),
        )
        for label, left, right, left_profile, right_profile in combinations:
            for mode in ("DIRECT", "FULL_CLOSURE"):
                probes.append(
                    execute_pair_probe(
                        java,
                        probe_classes,
                        oracle_jar,
                        work_root,
                        selected,
                        label,
                        mode,
                        left,
                        right,
                        left_profile,
                        right_profile,
                    )
                )

    case_classes = {
        case_id: case_classification(
            case_id, probes, discovered_results.get(case_id)
        )
        for case_id in ("C001", "C002")
    }
    report = {
        "schema": SCHEMA,
        "oracle": ORACLE,
        "source_path": {
            "snapshot_then_differential": "StructureDefinitionComparer.compare",
            "differential_navigation": "DefinitionNavigator(diff=true)",
            "local_slice_failure": "DefinitionNavigator.loadChildren/DN_SLICE_NO_DEFINITION",
        },
        "discoveries": discoveries,
        "discovery_probes": discovery_probes,
        "differential_shapes": shapes,
        "probes": probes,
        "context_mode_comparisons": context_mode_comparisons(probes),
        "case_classifications": case_classes,
        "root_cause_class": primary_classification(case_classes),
    }
    output.write_bytes(canonical_json_bytes(report))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    acquire_parser = subparsers.add_parser("acquire")
    acquire_parser.add_argument("--commandf", type=Path, required=True)
    acquire_parser.add_argument("--work-root", type=Path, required=True)
    acquire_parser.add_argument("--output", type=Path, required=True)
    qualify_parser = subparsers.add_parser("qualify")
    qualify_parser.add_argument("--java", type=Path, required=True)
    qualify_parser.add_argument("--probe-classes", type=Path, required=True)
    qualify_parser.add_argument("--oracle-jar", type=Path, required=True)
    qualify_parser.add_argument("--work-root", type=Path, required=True)
    qualify_parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.command == "acquire":
        acquire(args.commandf, args.work_root, args.output)
    else:
        qualify(
            args.java,
            args.probe_classes,
            args.oracle_jar,
            args.work_root,
            args.output,
        )


if __name__ == "__main__":
    main()
