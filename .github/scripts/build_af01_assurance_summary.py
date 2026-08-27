#!/usr/bin/env python3
"""Build the deterministic AF-01 assurance summary from exact-source evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import tomllib
from collections import Counter
from pathlib import Path
from typing import Any

CHECKOUT_ACTION = "fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09"
CARGO_DENY_ACTION = "3c6349835b2b7b196a839186cb8b78e02f7b5f25"
CARGO_DENY_VERSION = "0.20.2"
CARGO_AUDIT_VERSION = "0.22.2"
ZIZMOR_ACTION = "3dc1ecc9bcb9e94e9b2c709687979e1298497054"
ZIZMOR_VERSION = "1.29.0"
RUSTSEC_ORIGIN = "https://github.com/RustSec/advisory-db.git"
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
INVENTORY_COMMAND = ["cargo", "metadata", "--locked", "--format-version", "1"]
CONFIG_PATHS = (
    ".github/workflow-trust-policy.json",
    "Cargo.lock",
    "deny.toml",
)
EVIDENCE_FILES = {
    "workflow_trust": "workflow-trust.json",
    "dependency_inventory": "dependency-inventory.json",
    "dependency_inventory_proof": "dependency-inventory-proof.json",
    "cargo_deny": "cargo-deny-proof.json",
    "cargo_audit": "cargo-audit-proof.json",
    "cargo_audit_result": "cargo-audit.json",
    "zizmor": "zizmor-proof.json",
}
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")


class AssuranceError(ValueError):
    """Raised when evidence cannot prove the requested exact source state."""


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    if not path.is_file():
        raise AssuranceError(f"required file is missing: {path}")
    return sha256_bytes(path.read_bytes())


def read_json(path: Path) -> dict[str, Any]:
    if not path.is_file():
        raise AssuranceError(f"required evidence is missing: {path.name}")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise AssuranceError(f"invalid JSON evidence {path.name}: {error}") from error
    if not isinstance(value, dict):
        raise AssuranceError(f"evidence root must be an object: {path.name}")
    return value


def git(root: Path, *args: str) -> str:
    try:
        return subprocess.check_output(
            ["git", "-C", str(root), *args], text=True, stderr=subprocess.STDOUT
        ).strip()
    except subprocess.CalledProcessError as error:
        raise AssuranceError(f"git {' '.join(args)} failed: {error.output.strip()}") from error


def tracked_files(root: Path) -> list[str]:
    rendered = git(root, "ls-files", "-z")
    return sorted(path for path in rendered.split("\0") if path)


def security_surfaces(paths: list[str]) -> tuple[list[str], list[str]]:
    workflows = sorted(
        path
        for path in paths
        if path.startswith(".github/workflows/") and path.endswith((".yml", ".yaml"))
    )
    actions = sorted(
        path for path in paths if Path(path).name in {"action.yml", "action.yaml"}
    )
    return workflows, actions


def surface_digest(root: Path, paths: list[str]) -> str:
    digest = hashlib.sha256()
    for relative in paths:
        encoded = relative.encode("utf-8")
        data = (root / relative).read_bytes()
        digest.update(len(encoded).to_bytes(4, "big"))
        digest.update(encoded)
        digest.update(len(data).to_bytes(8, "big"))
        digest.update(data)
    return digest.hexdigest()


def canonical_json_sha256(value: object) -> str:
    rendered = json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")
    return sha256_bytes(rendered)


def canonical_graph_sha256(packages: list[dict[str, object]]) -> str:
    return canonical_json_sha256(packages)


def require_string(value: object, message: str) -> str:
    if not isinstance(value, str) or not value:
        raise AssuranceError(message)
    return value


def require_exact_source(root: Path, source_sha: str, tree_sha: str) -> None:
    if not HEX40.fullmatch(source_sha) or not HEX40.fullmatch(tree_sha):
        raise AssuranceError("source and tree identities must be lowercase 40-hex SHA-1 values")
    actual_source = git(root, "rev-parse", "HEAD")
    actual_tree = git(root, "rev-parse", "HEAD^{tree}")
    if actual_source != source_sha:
        raise AssuranceError(f"source SHA mismatch: expected {source_sha}, found {actual_source}")
    if actual_tree != tree_sha:
        raise AssuranceError(f"tree SHA mismatch: expected {tree_sha}, found {actual_tree}")
    dirty = git(root, "status", "--porcelain=v1", "--untracked-files=all")
    if dirty:
        raise AssuranceError(f"source worktree is dirty or has unexpected files: {dirty.splitlines()[0]}")


def validate_workflow_trust(
    evidence: dict[str, Any], workflows: list[str], actions: list[str]
) -> None:
    if evidence.get("schema") != 1 or evidence.get("ok") is not True:
        raise AssuranceError("workflow trust evidence is not a successful schema-1 audit")
    if evidence.get("findings") != []:
        raise AssuranceError("workflow trust evidence contains findings")
    if evidence.get("workflows") != workflows:
        raise AssuranceError("workflow trust evidence does not cover the exact workflow set")
    if evidence.get("action_metadata") != actions:
        raise AssuranceError("workflow trust evidence does not cover both exact Action metadata forms")


def validate_dependency_inventory(evidence: dict[str, Any]) -> str:
    if evidence.get("schema") != 2 or evidence.get("ok") is not True:
        raise AssuranceError("dependency inventory is not a successful schema-2 exact graph")
    if evidence.get("unknown_license") != []:
        raise AssuranceError("dependency inventory contains unknown third-party license metadata")

    packages = evidence.get("packages")
    if not isinstance(packages, list) or evidence.get("package_count") != len(packages):
        raise AssuranceError("dependency inventory package count is inconsistent")

    required_package_keys = {
        "dependencies",
        "license",
        "name",
        "package_id",
        "source",
        "source_class",
        "version",
        "workspace",
    }
    required_edge_keys = {"name", "package_id", "package_name", "source", "version"}
    package_by_id: dict[str, dict[str, object]] = {}
    source_classes: Counter[str] = Counter()

    for index, package in enumerate(packages):
        if not isinstance(package, dict) or set(package) != required_package_keys:
            raise AssuranceError(f"dependency inventory package {index} has an invalid schema")
        package_id = require_string(
            package.get("package_id"), f"dependency inventory package {index} has invalid identity"
        )
        require_string(package.get("name"), f"dependency inventory package {index} has invalid name")
        require_string(package.get("version"), f"dependency inventory package {index} has invalid version")
        if package_id in package_by_id:
            raise AssuranceError(f"dependency inventory contains duplicate package id: {package_id}")

        source = package.get("source")
        if source is not None and not isinstance(source, str):
            raise AssuranceError(f"dependency inventory package {package_id} has invalid source")
        license_expr = package.get("license")
        if license_expr is not None and not isinstance(license_expr, str):
            raise AssuranceError(f"dependency inventory package {package_id} has invalid license")
        workspace = package.get("workspace")
        if not isinstance(workspace, bool):
            raise AssuranceError(f"dependency inventory package {package_id} has invalid workspace flag")
        source_class = package.get("source_class")
        if source_class not in {"workspace", "crates.io", "other"}:
            raise AssuranceError(f"dependency inventory package {package_id} has invalid source class")
        if workspace != (source_class == "workspace"):
            raise AssuranceError(f"dependency inventory package {package_id} has inconsistent workspace class")
        if source_class == "crates.io" and source != CRATES_IO_SOURCE:
            raise AssuranceError(f"dependency inventory package {package_id} has inconsistent crates.io source")

        dependencies = package.get("dependencies")
        if not isinstance(dependencies, list):
            raise AssuranceError(f"dependency inventory package {package_id} has invalid dependency edges")
        package_by_id[package_id] = package
        source_classes[str(source_class)] += 1

    ordered_packages = sorted(
        packages,
        key=lambda item: (
            str(item["name"]),
            str(item["version"]),
            str(item["source"]),
            str(item["package_id"]),
        ),
    )
    if packages != ordered_packages:
        raise AssuranceError("dependency inventory package ordering is not canonical")

    for package in packages:
        package_id = str(package["package_id"])
        dependencies = package["dependencies"]
        seen_edges: set[tuple[str, str]] = set()
        for edge_index, edge in enumerate(dependencies):
            if not isinstance(edge, dict) or set(edge) != required_edge_keys:
                raise AssuranceError(
                    f"dependency edge {edge_index} for {package_id} has an invalid schema"
                )
            edge_name = require_string(
                edge.get("name"), f"dependency edge {edge_index} for {package_id} has invalid name"
            )
            target_id = require_string(
                edge.get("package_id"),
                f"dependency edge {edge_index} for {package_id} has invalid target id",
            )
            target = package_by_id.get(target_id)
            if target is None:
                raise AssuranceError(
                    f"dependency edge {edge_index} for {package_id} references unknown package id"
                )
            if (
                edge.get("package_name") != target["name"]
                or edge.get("version") != target["version"]
                or edge.get("source") != target["source"]
            ):
                raise AssuranceError(
                    f"dependency edge {edge_index} for {package_id} disagrees with target package identity"
                )
            identity = (edge_name, target_id)
            if identity in seen_edges:
                raise AssuranceError(f"dependency inventory contains duplicate edge for {package_id}")
            seen_edges.add(identity)
        ordered_edges = sorted(
            dependencies,
            key=lambda edge: (
                str(edge["name"]),
                str(edge["package_name"]),
                str(edge["version"]),
                str(edge["source"]),
                str(edge["package_id"]),
            ),
        )
        if dependencies != ordered_edges:
            raise AssuranceError(f"dependency edges for {package_id} are not canonically ordered")

    expected_source_classes = dict(sorted(source_classes.items()))
    if evidence.get("source_classes") != expected_source_classes:
        raise AssuranceError("dependency inventory source-class counts are inconsistent")

    graph_sha = evidence.get("graph_sha256")
    expected_graph_sha = canonical_graph_sha256(packages)
    if not isinstance(graph_sha, str) or not HEX64.fullmatch(graph_sha):
        raise AssuranceError("dependency inventory graph digest is missing or malformed")
    if graph_sha != expected_graph_sha:
        raise AssuranceError("dependency inventory graph digest does not match exact package records")
    return graph_sha


def validate_inventory_against_cargo_lock(
    root: Path, packages: list[dict[str, object]]
) -> str:
    try:
        lock = tomllib.loads((root / "Cargo.lock").read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
        raise AssuranceError(f"Cargo.lock cannot be parsed for package identity proof: {error}") from error
    lock_packages = lock.get("package")
    if not isinstance(lock_packages, list):
        raise AssuranceError("Cargo.lock is missing package records")

    canonical_lock: list[dict[str, object]] = []
    lock_by_identity: dict[tuple[str, str, object], dict[str, object]] = {}
    for index, package in enumerate(lock_packages):
        if not isinstance(package, dict):
            raise AssuranceError(f"Cargo.lock package {index} is not an object")
        name = require_string(package.get("name"), f"Cargo.lock package {index} has invalid name")
        version = require_string(
            package.get("version"), f"Cargo.lock package {index} has invalid version"
        )
        source = package.get("source")
        if source is not None and not isinstance(source, str):
            raise AssuranceError(f"Cargo.lock package {name}@{version} has invalid source")
        checksum = package.get("checksum")
        if checksum is not None and (
            not isinstance(checksum, str) or HEX64.fullmatch(checksum) is None
        ):
            raise AssuranceError(f"Cargo.lock package {name}@{version} has invalid checksum")
        if source == CRATES_IO_SOURCE and checksum is None:
            raise AssuranceError(f"Cargo.lock crates.io package {name}@{version} is missing checksum")
        identity = (name, version, source)
        if identity in lock_by_identity:
            raise AssuranceError(f"Cargo.lock contains duplicate package identity: {name}@{version}")
        record = {
            "checksum": checksum,
            "name": name,
            "source": source,
            "version": version,
        }
        lock_by_identity[identity] = record
        canonical_lock.append(record)

    inventory_identities = {
        (str(package["name"]), str(package["version"]), package["source"])
        for package in packages
    }
    lock_identities = set(lock_by_identity)
    if inventory_identities != lock_identities:
        missing = sorted(str(value) for value in lock_identities - inventory_identities)
        extra = sorted(str(value) for value in inventory_identities - lock_identities)
        raise AssuranceError(
            f"dependency inventory does not match Cargo.lock package identities: missing={missing[:1]} extra={extra[:1]}"
        )

    canonical_lock.sort(
        key=lambda item: (str(item["name"]), str(item["version"]), str(item["source"]))
    )
    return canonical_json_sha256(canonical_lock)


def require_head(proof: dict[str, Any], source_sha: str, label: str) -> None:
    if proof.get("schema") != 1 or proof.get("head_sha") != source_sha:
        raise AssuranceError(f"{label} proof is not bound to the exact source SHA")


def validate_dependency_inventory_proof(
    proof: dict[str, Any],
    source_sha: str,
    cargo_lock_sha: str,
    inventory_sha: str,
    graph_sha: str,
) -> None:
    require_head(proof, source_sha, "dependency inventory")
    if (
        proof.get("command") != INVENTORY_COMMAND
        or proof.get("cargo_lock_sha256") != cargo_lock_sha
        or proof.get("inventory_sha256") != inventory_sha
        or proof.get("graph_sha256") != graph_sha
    ):
        raise AssuranceError("dependency inventory proof identity/graph mismatch")


def validate_cargo_audit_result(
    result: dict[str, Any], proof: dict[str, Any], package_count: int
) -> None:
    vulnerabilities = result.get("vulnerabilities")
    if not isinstance(vulnerabilities, dict):
        raise AssuranceError("cargo-audit result is missing vulnerabilities evidence")
    if vulnerabilities.get("found") is not False:
        raise AssuranceError("cargo-audit result does not explicitly prove zero vulnerabilities")
    if vulnerabilities.get("count") != 0 or vulnerabilities.get("list") != []:
        raise AssuranceError("cargo-audit zero-vulnerability fields are inconsistent")

    lockfile = result.get("lockfile")
    if not isinstance(lockfile, dict) or lockfile.get("dependency-count") != package_count:
        raise AssuranceError("cargo-audit lockfile dependency count does not match exact inventory")

    database = result.get("database")
    if not isinstance(database, dict):
        raise AssuranceError("cargo-audit result is missing advisory database evidence")
    advisory_count = database.get("advisory-count")
    if not isinstance(advisory_count, int) or isinstance(advisory_count, bool) or advisory_count < 0:
        raise AssuranceError("cargo-audit advisory database count is invalid")
    if database.get("last-commit") != proof.get("advisory_db_commit"):
        raise AssuranceError("cargo-audit result advisory database commit does not match proof")
    if not isinstance(database.get("last-updated"), str) or not database["last-updated"]:
        raise AssuranceError("cargo-audit result advisory database timestamp is invalid")
    if not isinstance(result.get("settings"), dict) or not isinstance(result.get("warnings"), dict):
        raise AssuranceError("cargo-audit result is missing settings/warnings objects")


def build_summary(root: Path, evidence_dir: Path, source_sha: str, tree_sha: str) -> dict[str, Any]:
    root = root.resolve()
    evidence_dir = evidence_dir.resolve()
    require_exact_source(root, source_sha, tree_sha)

    paths = tracked_files(root)
    workflows, actions = security_surfaces(paths)
    workflow_trust = read_json(evidence_dir / EVIDENCE_FILES["workflow_trust"])
    dependency_inventory = read_json(evidence_dir / EVIDENCE_FILES["dependency_inventory"])
    dependency_inventory_proof = read_json(
        evidence_dir / EVIDENCE_FILES["dependency_inventory_proof"]
    )
    cargo_deny = read_json(evidence_dir / EVIDENCE_FILES["cargo_deny"])
    cargo_audit = read_json(evidence_dir / EVIDENCE_FILES["cargo_audit"])
    cargo_audit_result = read_json(evidence_dir / EVIDENCE_FILES["cargo_audit_result"])
    zizmor = read_json(evidence_dir / EVIDENCE_FILES["zizmor"])

    validate_workflow_trust(workflow_trust, workflows, actions)
    graph_sha = validate_dependency_inventory(dependency_inventory)
    lock_packages_sha = validate_inventory_against_cargo_lock(
        root, dependency_inventory["packages"]
    )

    cargo_lock_sha = sha256_file(root / "Cargo.lock")
    deny_sha = sha256_file(root / "deny.toml")
    inventory_sha = sha256_file(evidence_dir / EVIDENCE_FILES["dependency_inventory"])
    validate_dependency_inventory_proof(
        dependency_inventory_proof,
        source_sha,
        cargo_lock_sha,
        inventory_sha,
        graph_sha,
    )

    require_head(cargo_deny, source_sha, "cargo-deny")
    if (
        cargo_deny.get("action_commit") != CARGO_DENY_ACTION
        or cargo_deny.get("cargo_deny_version") != CARGO_DENY_VERSION
        or cargo_deny.get("cargo_lock_sha256") != cargo_lock_sha
        or cargo_deny.get("deny_toml_sha256") != deny_sha
        or cargo_deny.get("checks") != ["advisories", "bans", "licenses", "sources"]
    ):
        raise AssuranceError("cargo-deny proof identity/configuration mismatch")

    require_head(cargo_audit, source_sha, "cargo-audit")
    if (
        cargo_audit.get("cargo_audit_version") != CARGO_AUDIT_VERSION
        or cargo_audit.get("cargo_lock_sha256") != cargo_lock_sha
        or cargo_audit.get("exit_code") != 0
        or cargo_audit.get("advisory_db_origin") != RUSTSEC_ORIGIN
        or not HEX40.fullmatch(str(cargo_audit.get("advisory_db_commit", "")))
    ):
        raise AssuranceError("cargo-audit proof identity/result mismatch")
    validate_cargo_audit_result(
        cargo_audit_result, cargo_audit, int(dependency_inventory["package_count"])
    )

    require_head(zizmor, source_sha, "zizmor")
    if (
        zizmor.get("action_commit") != ZIZMOR_ACTION
        or zizmor.get("zizmor_version") != ZIZMOR_VERSION
        or zizmor.get("min_severity") != "medium"
        or zizmor.get("online_audits") is not False
        or zizmor.get("advanced_security") is not False
    ):
        raise AssuranceError("zizmor proof identity/policy mismatch")

    config = {path: sha256_file(root / path) for path in CONFIG_PATHS}
    surface_paths = sorted(set(workflows + actions))
    evidence_sha256 = {
        key: sha256_file(evidence_dir / filename)
        for key, filename in sorted(EVIDENCE_FILES.items())
    }

    return {
        "config_sha256": config,
        "dependency_graph": {
            "cargo_lock_packages_sha256": lock_packages_sha,
            "graph_sha256": graph_sha,
            "package_count": dependency_inventory["package_count"],
            "schema": dependency_inventory["schema"],
        },
        "evidence_sha256": evidence_sha256,
        "execution_identities": {
            "actions": {
                "actions/checkout": CHECKOUT_ACTION,
                "EmbarkStudios/cargo-deny-action": CARGO_DENY_ACTION,
                "zizmorcore/zizmor-action": ZIZMOR_ACTION,
            },
            "containers": [],
            "tools": {
                "cargo-audit": CARGO_AUDIT_VERSION,
                "cargo-deny": CARGO_DENY_VERSION,
                "zizmor": ZIZMOR_VERSION,
            },
        },
        "rustsec_advisory_db_commit": cargo_audit["advisory_db_commit"],
        "schema": 1,
        "source": {"sha": source_sha, "tree": tree_sha},
        "workflow_surface": {
            "action_metadata": actions,
            "sha256": surface_digest(root, surface_paths),
            "workflows": workflows,
        },
    }


def render_summary(summary: dict[str, Any]) -> bytes:
    return (json.dumps(summary, indent=2, sort_keys=True, separators=(",", ": ")) + "\n").encode(
        "utf-8"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--evidence-dir", type=Path, required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--tree-sha", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        summary = build_summary(args.root, args.evidence_dir, args.source_sha, args.tree_sha)
        rendered = render_summary(summary)
        args.output.write_bytes(rendered)
    except (AssuranceError, OSError, UnicodeError) as error:
        print(json.dumps({"error": str(error), "ok": False}, sort_keys=True))
        return 1
    digest = sha256_bytes(rendered)
    print(f"AF01_ASSURANCE_SHA256={digest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
