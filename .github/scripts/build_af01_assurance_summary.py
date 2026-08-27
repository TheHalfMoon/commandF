#!/usr/bin/env python3
"""Build the deterministic AF-01 assurance summary from exact-source evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path
from typing import Any

CHECKOUT_ACTION = "fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09"
CARGO_DENY_ACTION = "3c6349835b2b7b196a839186cb8b78e02f7b5f25"
CARGO_DENY_VERSION = "0.20.2"
CARGO_AUDIT_VERSION = "0.22.2"
ZIZMOR_ACTION = "3dc1ecc9bcb9e94e9b2c709687979e1298497054"
ZIZMOR_VERSION = "1.29.0"
RUSTSEC_ORIGIN = "https://github.com/RustSec/advisory-db.git"
CONFIG_PATHS = (
    ".github/workflow-trust-policy.json",
    "Cargo.lock",
    "deny.toml",
)
EVIDENCE_FILES = {
    "workflow_trust": "workflow-trust.json",
    "dependency_inventory": "dependency-inventory.json",
    "cargo_deny": "cargo-deny-proof.json",
    "cargo_audit": "cargo-audit-proof.json",
    "cargo_audit_result": "cargo-audit.json",
    "zizmor": "zizmor-proof.json",
}
HEX40 = re.compile(r"^[0-9a-f]{40}$")


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


def validate_dependency_inventory(evidence: dict[str, Any]) -> None:
    if evidence.get("schema") != 2 or evidence.get("ok") is not True:
        raise AssuranceError("dependency inventory is not a successful schema-2 exact graph")
    if evidence.get("unknown_license") != []:
        raise AssuranceError("dependency inventory contains unknown third-party license metadata")
    if not isinstance(evidence.get("packages"), list) or evidence.get("package_count") != len(
        evidence["packages"]
    ):
        raise AssuranceError("dependency inventory package count is inconsistent")


def require_head(proof: dict[str, Any], source_sha: str, label: str) -> None:
    if proof.get("schema") != 1 or proof.get("head_sha") != source_sha:
        raise AssuranceError(f"{label} proof is not bound to the exact source SHA")


def build_summary(root: Path, evidence_dir: Path, source_sha: str, tree_sha: str) -> dict[str, Any]:
    root = root.resolve()
    evidence_dir = evidence_dir.resolve()
    require_exact_source(root, source_sha, tree_sha)

    paths = tracked_files(root)
    workflows, actions = security_surfaces(paths)
    workflow_trust = read_json(evidence_dir / EVIDENCE_FILES["workflow_trust"])
    dependency_inventory = read_json(evidence_dir / EVIDENCE_FILES["dependency_inventory"])
    cargo_deny = read_json(evidence_dir / EVIDENCE_FILES["cargo_deny"])
    cargo_audit = read_json(evidence_dir / EVIDENCE_FILES["cargo_audit"])
    cargo_audit_result = read_json(evidence_dir / EVIDENCE_FILES["cargo_audit_result"])
    zizmor = read_json(evidence_dir / EVIDENCE_FILES["zizmor"])

    validate_workflow_trust(workflow_trust, workflows, actions)
    validate_dependency_inventory(dependency_inventory)

    cargo_lock_sha = sha256_file(root / "Cargo.lock")
    deny_sha = sha256_file(root / "deny.toml")

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
    if cargo_audit_result.get("vulnerabilities", {}).get("found") is True:
        raise AssuranceError("cargo-audit result reports RustSec vulnerabilities")

    require_head(zizmor, source_sha, "zizmor")
    if (
        zizmor.get("action_commit") != ZIZMOR_ACTION
        or zizmor.get("zizmor_version") != ZIZMOR_VERSION
        or zizmor.get("min_severity") != "medium"
        or zizmor.get("online_audits") is not False
        or zizmor.get("advanced_security") is not False
    ):
        raise AssuranceError("zizmor proof identity/policy mismatch")

    config = {
        path: sha256_file(root / path)
        for path in CONFIG_PATHS
    }
    surface_paths = sorted(set(workflows + actions))
    evidence_sha256 = {
        key: sha256_file(evidence_dir / filename)
        for key, filename in sorted(EVIDENCE_FILES.items())
    }

    return {
        "config_sha256": config,
        "dependency_graph": {
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
