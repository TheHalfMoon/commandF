#!/usr/bin/env python3
"""Build final AF-01 assurance with fetched-crate and executed-scanner binding."""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import sys
from pathlib import Path
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
FETCH_COMMAND = ["cargo", "fetch", "--locked"]
CHECKSUM_EVIDENCE = "crate-checksums.json"
ASSURANCE_WORKFLOW = Path(".github/workflows/af01-assurance-proof.yml")


def _load_module(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, SCRIPT_DIR / filename)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {filename}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


BASE = _load_module("af01_assurance_summary_core", "build_af01_assurance_summary.py")
VERIFY = _load_module("af01_crate_checksum_verifier", "verify_crate_checksums.py")
SCANNERS = _load_module("af01_scanner_contract", "validate_af01_scanner_invocations.py")


def validate_checksum_binding(
    root: Path,
    evidence_dir: Path,
    cache_root: Path,
) -> tuple[str, int]:
    root = root.resolve()
    evidence_dir = evidence_dir.resolve()
    cache_root = cache_root.resolve()

    inventory_path = evidence_dir / BASE.EVIDENCE_FILES["dependency_inventory"]
    proof_path = evidence_dir / BASE.EVIDENCE_FILES["dependency_inventory_proof"]
    checksum_path = evidence_dir / CHECKSUM_EVIDENCE

    inventory = BASE.read_json(inventory_path)
    proof = BASE.read_json(proof_path)
    recorded = BASE.read_json(checksum_path)

    try:
        regenerated = VERIFY.verify(
            inventory,
            VERIFY.load_lock(root / "Cargo.lock"),
            cache_root,
        )
    except VERIFY.ChecksumError as error:
        raise BASE.AssuranceError(f"fetched crate checksum verification failed: {error}") from error

    if recorded != regenerated:
        raise BASE.AssuranceError(
            "recorded crate checksum evidence does not match independently reverified archive bytes"
        )

    checksums = recorded.get("checksums")
    package_count = recorded.get("package_count")
    if (
        recorded.get("schema") != 1
        or recorded.get("ok") is not True
        or not isinstance(checksums, dict)
        or not isinstance(package_count, int)
        or isinstance(package_count, bool)
        or package_count != len(checksums)
    ):
        raise BASE.AssuranceError("crate checksum evidence is not a successful canonical schema-1 proof")

    evidence_sha = BASE.sha256_file(checksum_path)
    if (
        proof.get("fetch_command") != FETCH_COMMAND
        or proof.get("crate_checksums_sha256") != evidence_sha
        or proof.get("crate_checksum_package_count") != package_count
    ):
        raise BASE.AssuranceError("dependency inventory proof does not bind exact crate checksum evidence")

    return evidence_sha, package_count


def validate_scanner_binding(root: Path, evidence_dir: Path) -> dict[str, Any]:
    root = root.resolve()
    evidence_dir = evidence_dir.resolve()
    try:
        contract = SCANNERS.validate_workflow(root / ASSURANCE_WORKFLOW)
    except SCANNERS.ScannerContractError as error:
        raise BASE.AssuranceError(f"executed scanner contract validation failed: {error}") from error

    cargo_deny = BASE.read_json(evidence_dir / BASE.EVIDENCE_FILES["cargo_deny"])
    cargo_audit = BASE.read_json(evidence_dir / BASE.EVIDENCE_FILES["cargo_audit"])
    zizmor = BASE.read_json(evidence_dir / BASE.EVIDENCE_FILES["zizmor"])

    deny_contract = contract.get("cargo_deny")
    audit_contract = contract.get("cargo_audit")
    zizmor_contract = contract.get("zizmor")
    if not all(isinstance(value, dict) for value in (deny_contract, audit_contract, zizmor_contract)):
        raise BASE.AssuranceError("executed scanner contract has an invalid schema")

    deny_inputs = deny_contract["inputs"]
    zizmor_inputs = zizmor_contract["inputs"]
    if not isinstance(deny_inputs, dict) or not isinstance(zizmor_inputs, dict):
        raise BASE.AssuranceError("executed scanner inputs have an invalid schema")

    deny_revision = str(deny_contract["uses"]).rsplit("@", 1)[-1]
    zizmor_revision = str(zizmor_contract["uses"]).rsplit("@", 1)[-1]
    deny_checks = str(deny_inputs.get("command-arguments", "")).split()

    if (
        cargo_deny.get("action_commit") != deny_revision
        or cargo_deny.get("checks") != deny_checks
    ):
        raise BASE.AssuranceError("cargo-deny proof does not match the executed action invocation")

    if cargo_audit.get("cargo_audit_version") != audit_contract.get("version"):
        raise BASE.AssuranceError("cargo-audit proof does not match the executed install/run contract")

    if (
        zizmor.get("action_commit") != zizmor_revision
        or zizmor.get("zizmor_version") != zizmor_inputs.get("version")
        or zizmor.get("min_severity") != zizmor_inputs.get("min-severity")
        or zizmor.get("online_audits") is not (zizmor_inputs.get("online-audits") == "true")
        or zizmor.get("advanced_security") is not (zizmor_inputs.get("advanced-security") == "true")
    ):
        raise BASE.AssuranceError("zizmor proof does not match the executed action invocation")

    return contract


def build_verified_summary(
    root: Path,
    evidence_dir: Path,
    source_sha: str,
    tree_sha: str,
    cache_root: Path,
) -> dict[str, Any]:
    summary = BASE.build_summary(root, evidence_dir, source_sha, tree_sha)
    checksum_sha, package_count = validate_checksum_binding(root, evidence_dir, cache_root)
    scanner_contract = validate_scanner_binding(root, evidence_dir)

    dependency_graph = summary.get("dependency_graph")
    evidence_sha256 = summary.get("evidence_sha256")
    execution_identities = summary.get("execution_identities")
    if (
        not isinstance(dependency_graph, dict)
        or not isinstance(evidence_sha256, dict)
        or not isinstance(execution_identities, dict)
    ):
        raise BASE.AssuranceError("core assurance summary has an invalid dependency/evidence schema")

    dependency_graph["crate_checksum_package_count"] = package_count
    dependency_graph["crate_checksums_sha256"] = checksum_sha
    dependency_graph["fetch_command"] = FETCH_COMMAND
    evidence_sha256["crate_checksums"] = checksum_sha
    execution_identities["scanner_contract"] = scanner_contract
    execution_identities["scanner_invocations_sha256"] = scanner_contract["sha256"]
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--evidence-dir", type=Path, required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--tree-sha", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument(
        "--cache-root",
        type=Path,
        default=Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))),
    )
    args = parser.parse_args()

    try:
        summary = build_verified_summary(
            args.root,
            args.evidence_dir,
            args.source_sha,
            args.tree_sha,
            args.cache_root,
        )
        rendered = BASE.render_summary(summary)
        args.output.write_bytes(rendered)
    except (BASE.AssuranceError, OSError, UnicodeError) as error:
        print(json.dumps({"error": str(error), "ok": False}, sort_keys=True))
        return 1

    digest = BASE.sha256_bytes(rendered)
    print(f"AF01_ASSURANCE_SHA256={digest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
