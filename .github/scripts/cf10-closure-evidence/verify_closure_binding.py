#!/usr/bin/env python3
"""Verify CF-10 durable lockfile evidence binds exactly to summary closure evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

SCHEMA = 1
SIDES = ("before", "after")


class BindingError(ValueError):
    pass


def _require_dict(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise BindingError(f"{label} must be an object")
    return value


def _require_list(value: Any, label: str) -> list[Any]:
    if not isinstance(value, list):
        raise BindingError(f"{label} must be an array")
    return value


def _require_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise BindingError(f"{label} must be a non-empty string")
    return value


def _require_sha256(value: Any, label: str) -> str:
    text = _require_string(value, label)
    if len(text) != 64 or any(char not in "0123456789abcdef" for char in text):
        raise BindingError(f"{label} must be a lowercase SHA-256 hex digest")
    return text


def _load_json(path: Path, label: str) -> dict[str, Any]:
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise BindingError(f"{label} is unreadable: {path}: {error}") from error
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as error:
        raise BindingError(f"{label} is invalid JSON: {path}: {error}") from error
    return _require_dict(value, label)


def _normalize_dependencies(value: Any, label: str) -> dict[str, str]:
    dependencies = _require_dict(value, label)
    normalized: dict[str, str] = {}
    for name in sorted(dependencies):
        dep_name = _require_string(name, f"{label} key")
        dep_version = _require_string(dependencies[name], f"{label}[{dep_name!r}]")
        normalized[dep_name] = dep_version
    return normalized


def _normalize_package(value: Any, label: str) -> dict[str, Any]:
    package = _require_dict(value, label)
    return {
        "name": _require_string(package.get("name"), f"{label}.name"),
        "version": _require_string(package.get("version"), f"{label}.version"),
        "sha256": _require_sha256(package.get("sha256"), f"{label}.sha256"),
        "dependencies": _normalize_dependencies(
            package.get("dependencies"), f"{label}.dependencies"
        ),
    }


def _package_sort_key(package: dict[str, Any]) -> tuple[Any, ...]:
    return (
        package["name"],
        package["version"],
        package["sha256"],
        tuple(package["dependencies"].items()),
    )


def normalize_lock_closure(lockfile: dict[str, Any], label: str) -> list[dict[str, Any]]:
    if lockfile.get("schema") != SCHEMA:
        raise BindingError(f"{label}.schema must be {SCHEMA}")
    packages = _require_list(lockfile.get("packages"), f"{label}.packages")
    closure = [
        _normalize_package(value, f"{label}.packages[{index}]")
        for index, value in enumerate(packages)
    ]
    closure.sort(key=_package_sort_key)
    return closure


def closure_bytes(closure: list[dict[str, Any]]) -> bytes:
    # Mirrors serde_json::to_vec(Vec<CorpusClosurePackage>): struct field insertion
    # order is name, version, sha256, dependencies; BTreeMap dependency keys are sorted.
    return json.dumps(
        closure,
        ensure_ascii=False,
        separators=(",", ":"),
    ).encode("utf-8")


def closure_sha256(closure: list[dict[str, Any]]) -> str:
    return hashlib.sha256(closure_bytes(closure)).hexdigest()


def _assert_root_identity(
    lockfile: dict[str, Any],
    *,
    package_name: str,
    version: str,
    sha256: str,
    label: str,
) -> None:
    roots = _require_list(lockfile.get("roots"), f"{label}.roots")
    expected_root = f"{package_name}@{version}"
    if roots.count(expected_root) != 1:
        raise BindingError(
            f"{label}.roots must contain exactly one {expected_root!r}"
        )

    closure = normalize_lock_closure(lockfile, label)
    matches = [
        package
        for package in closure
        if package["name"] == package_name
        and package["version"] == version
        and package["sha256"] == sha256
    ]
    if len(matches) != 1:
        raise BindingError(
            f"{label} must contain exactly one root package identity "
            f"{package_name}@{version}#{sha256}"
        )


def verify_side(
    *,
    summary_case: dict[str, Any],
    side: str,
    lock_path: Path,
) -> None:
    case_id = _require_string(summary_case.get("case_id"), "summary case_id")
    package_name = _require_string(
        summary_case.get("package"), f"{case_id}.package"
    )
    summary_state = _require_dict(
        summary_case.get(side), f"{case_id}.{side}"
    )
    version = _require_string(
        summary_state.get("version"), f"{case_id}.{side}.version"
    )
    sha256 = _require_sha256(
        summary_state.get("sha256"), f"{case_id}.{side}.sha256"
    )
    expected_digest = _require_sha256(
        summary_state.get("closure_sha256"),
        f"{case_id}.{side}.closure_sha256",
    )
    summary_closure_raw = _require_list(
        summary_state.get("closure"), f"{case_id}.{side}.closure"
    )
    summary_closure = [
        _normalize_package(value, f"{case_id}.{side}.closure[{index}]")
        for index, value in enumerate(summary_closure_raw)
    ]
    if summary_closure != sorted(summary_closure, key=_package_sort_key):
        raise BindingError(f"{case_id}.{side}.closure is not canonically sorted")

    lockfile = _load_json(lock_path, f"{case_id}.{side} retained lockfile")
    _assert_root_identity(
        lockfile,
        package_name=package_name,
        version=version,
        sha256=sha256,
        label=f"{case_id}.{side} retained lockfile",
    )
    retained_closure = normalize_lock_closure(
        lockfile, f"{case_id}.{side} retained lockfile"
    )

    if retained_closure != summary_closure:
        raise BindingError(
            f"{case_id}.{side} retained lockfile closure does not match summary closure"
        )

    retained_digest = closure_sha256(retained_closure)
    if retained_digest != expected_digest:
        raise BindingError(
            f"{case_id}.{side} closure digest mismatch: "
            f"retained={retained_digest} summary={expected_digest}"
        )


def verify_summary(summary_path: Path, evidence_root: Path) -> None:
    summary = _load_json(summary_path, "summary")
    if summary.get("schema") != SCHEMA:
        raise BindingError(f"summary.schema must be {SCHEMA}")
    cases = _require_list(summary.get("cases"), "summary.cases")
    if not cases:
        raise BindingError("summary.cases must not be empty")

    seen_case_ids: set[str] = set()
    for index, raw_case in enumerate(cases):
        summary_case = _require_dict(raw_case, f"summary.cases[{index}]")
        case_id = _require_string(
            summary_case.get("case_id"), f"summary.cases[{index}].case_id"
        )
        if case_id in seen_case_ids:
            raise BindingError(f"duplicate summary case_id {case_id!r}")
        seen_case_ids.add(case_id)

        for side in SIDES:
            verify_side(
                summary_case=summary_case,
                side=side,
                lock_path=evidence_root / case_id / f"{side}.commandf.lock",
            )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--summary", type=Path, required=True)
    parser.add_argument("--evidence-root", type=Path, required=True)
    args = parser.parse_args(argv)

    try:
        verify_summary(args.summary, args.evidence_root)
    except BindingError as error:
        print(f"CF10_CLOSURE_BINDING_FAILED: {error}", file=sys.stderr)
        return 1

    print("CF10_CLOSURE_BINDING_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
