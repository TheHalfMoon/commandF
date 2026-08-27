#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("build_af01_assurance_summary.py")
SPEC = importlib.util.spec_from_file_location("af01_assurance_summary", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
SUMMARY = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = SUMMARY
SPEC.loader.exec_module(SUMMARY)


def run(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(root), *args], text=True).strip()


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


class AssuranceSummaryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        base = Path(self.temp.name)
        self.root = base / "repo"
        self.evidence = base / "evidence"
        self.root.mkdir()
        self.evidence.mkdir()

        files = {
            ".github/workflows/ci.yml": "name: ci\non: [pull_request]\n",
            ".github/workflow-trust-policy.json": "{\"schema\":1}\n",
            "action.yaml": "name: fixture\nruns:\n  using: composite\n  steps: []\n",
            "Cargo.lock": (
                "version = 4\n\n"
                "[[package]]\n"
                "name = \"commandf\"\n"
                "version = \"0.1.0\"\n\n"
                "[[package]]\n"
                "name = \"dep\"\n"
                "version = \"1.0.0\"\n"
                f"source = \"{SUMMARY.CRATES_IO_SOURCE}\"\n"
                f"checksum = \"{'b' * 64}\"\n"
            ),
            "deny.toml": "[bans]\nwildcards = \"deny\"\n",
        }
        for relative, content in files.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")

        subprocess.check_call(["git", "init", "-b", "main", str(self.root)], stdout=subprocess.DEVNULL)
        subprocess.check_call(["git", "-C", str(self.root), "config", "user.email", "af01@example.invalid"])
        subprocess.check_call(["git", "-C", str(self.root), "config", "user.name", "AF-01 Test"])
        subprocess.check_call(["git", "-C", str(self.root), "add", "."])
        subprocess.check_call(["git", "-C", str(self.root), "commit", "-m", "fixture"], stdout=subprocess.DEVNULL)
        self.source = run(self.root, "rev-parse", "HEAD")
        self.tree = run(self.root, "rev-parse", "HEAD^{tree}")
        self._write_valid_evidence()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def _sha(self, relative: str) -> str:
        return hashlib.sha256((self.root / relative).read_bytes()).hexdigest()

    def _inventory_packages(self) -> list[dict[str, object]]:
        dep_id = f"{SUMMARY.CRATES_IO_SOURCE}#dep@1.0.0"
        root_id = "path+file:///workspace/commandf#0.1.0"
        return [
            {
                "dependencies": [
                    {
                        "name": "dep",
                        "package_id": dep_id,
                        "package_name": "dep",
                        "source": SUMMARY.CRATES_IO_SOURCE,
                        "version": "1.0.0",
                    }
                ],
                "license": None,
                "name": "commandf",
                "package_id": root_id,
                "source": None,
                "source_class": "workspace",
                "version": "0.1.0",
                "workspace": True,
            },
            {
                "dependencies": [],
                "license": "MIT",
                "name": "dep",
                "package_id": dep_id,
                "source": SUMMARY.CRATES_IO_SOURCE,
                "source_class": "crates.io",
                "version": "1.0.0",
                "workspace": False,
            },
        ]

    def _write_inventory(self, packages: list[dict[str, object]] | None = None) -> None:
        packages = self._inventory_packages() if packages is None else packages
        inventory = {
            "schema": 2,
            "ok": True,
            "package_count": len(packages),
            "packages": packages,
            "graph_sha256": SUMMARY.canonical_graph_sha256(packages),
            "source_classes": {"crates.io": 1, "workspace": 1},
            "unknown_license": [],
        }
        write_json(self.evidence / "dependency-inventory.json", inventory)
        write_json(
            self.evidence / "dependency-inventory-proof.json",
            {
                "schema": 1,
                "head_sha": self.source,
                "command": SUMMARY.INVENTORY_COMMAND,
                "cargo_lock_sha256": self._sha("Cargo.lock"),
                "inventory_sha256": hashlib.sha256(
                    (self.evidence / "dependency-inventory.json").read_bytes()
                ).hexdigest(),
                "graph_sha256": inventory["graph_sha256"],
            },
        )

    def _write_valid_evidence(self) -> None:
        write_json(
            self.evidence / "workflow-trust.json",
            {
                "schema": 1,
                "ok": True,
                "workflows": [".github/workflows/ci.yml"],
                "action_metadata": ["action.yaml"],
                "findings": [],
            },
        )
        self._write_inventory()
        write_json(
            self.evidence / "cargo-deny-proof.json",
            {
                "schema": 1,
                "head_sha": self.source,
                "action_commit": SUMMARY.CARGO_DENY_ACTION,
                "cargo_deny_version": SUMMARY.CARGO_DENY_VERSION,
                "cargo_lock_sha256": self._sha("Cargo.lock"),
                "deny_toml_sha256": self._sha("deny.toml"),
                "checks": ["advisories", "bans", "licenses", "sources"],
            },
        )
        write_json(
            self.evidence / "cargo-audit-proof.json",
            {
                "schema": 1,
                "head_sha": self.source,
                "cargo_audit_version": SUMMARY.CARGO_AUDIT_VERSION,
                "cargo_lock_sha256": self._sha("Cargo.lock"),
                "exit_code": 0,
                "advisory_db_origin": SUMMARY.RUSTSEC_ORIGIN,
                "advisory_db_commit": "a" * 40,
            },
        )
        write_json(
            self.evidence / "cargo-audit.json",
            {
                "database": {
                    "advisory-count": 1,
                    "last-commit": "a" * 40,
                    "last-updated": "2026-08-27T00:00:00Z",
                },
                "lockfile": {"dependency-count": 2},
                "settings": {},
                "vulnerabilities": {"found": False, "count": 0, "list": []},
                "warnings": {},
            },
        )
        write_json(
            self.evidence / "zizmor-proof.json",
            {
                "schema": 1,
                "head_sha": self.source,
                "action_commit": SUMMARY.ZIZMOR_ACTION,
                "zizmor_version": SUMMARY.ZIZMOR_VERSION,
                "min_severity": "medium",
                "online_audits": False,
                "advanced_security": False,
            },
        )

    def build(self) -> dict[str, object]:
        return SUMMARY.build_summary(self.root, self.evidence, self.source, self.tree)

    def test_repeated_summary_is_byte_identical(self) -> None:
        first = SUMMARY.render_summary(self.build())
        second = SUMMARY.render_summary(self.build())
        self.assertEqual(first, second)
        self.assertEqual(SUMMARY.sha256_bytes(first), SUMMARY.sha256_bytes(second))

    def test_source_sha_mismatch_fails_closed(self) -> None:
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "source SHA mismatch"):
            SUMMARY.build_summary(self.root, self.evidence, "0" * 40, self.tree)

    def test_tree_sha_mismatch_fails_closed(self) -> None:
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "tree SHA mismatch"):
            SUMMARY.build_summary(self.root, self.evidence, self.source, "0" * 40)

    def test_missing_required_evidence_fails_closed(self) -> None:
        (self.evidence / "dependency-inventory-proof.json").unlink()
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "required evidence is missing"):
            self.build()

    def test_malformed_evidence_fails_closed(self) -> None:
        (self.evidence / "dependency-inventory.json").write_text("{", encoding="utf-8")
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "invalid JSON evidence"):
            self.build()

    def test_dependency_graph_digest_mismatch_fails_closed(self) -> None:
        inventory = json.loads(
            (self.evidence / "dependency-inventory.json").read_text(encoding="utf-8")
        )
        inventory["packages"][1]["version"] = "9.9.9"
        write_json(self.evidence / "dependency-inventory.json", inventory)
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "graph digest does not match"):
            self.build()

    def test_dependency_inventory_proof_must_bind_cargo_lock(self) -> None:
        proof = json.loads(
            (self.evidence / "dependency-inventory-proof.json").read_text(encoding="utf-8")
        )
        proof["cargo_lock_sha256"] = "0" * 64
        write_json(self.evidence / "dependency-inventory-proof.json", proof)
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "identity/graph mismatch"):
            self.build()

    def test_dependency_inventory_must_match_cargo_lock_identities(self) -> None:
        packages = self._inventory_packages()
        packages[1]["version"] = "2.0.0"
        self._write_inventory(packages)
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "does not match Cargo.lock"):
            self.build()

    def test_cargo_audit_empty_object_fails_closed(self) -> None:
        write_json(self.evidence / "cargo-audit.json", {})
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "missing vulnerabilities evidence"):
            self.build()

    def test_cargo_audit_missing_found_fails_closed(self) -> None:
        result = json.loads((self.evidence / "cargo-audit.json").read_text(encoding="utf-8"))
        result["vulnerabilities"].pop("found")
        write_json(self.evidence / "cargo-audit.json", result)
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "explicitly prove zero"):
            self.build()

    def test_cargo_audit_zero_fields_must_be_consistent(self) -> None:
        result = json.loads((self.evidence / "cargo-audit.json").read_text(encoding="utf-8"))
        result["vulnerabilities"]["count"] = 1
        write_json(self.evidence / "cargo-audit.json", result)
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "zero-vulnerability fields"):
            self.build()

    def test_cargo_audit_database_commit_must_match_proof(self) -> None:
        result = json.loads((self.evidence / "cargo-audit.json").read_text(encoding="utf-8"))
        result["database"]["last-commit"] = "b" * 40
        write_json(self.evidence / "cargo-audit.json", result)
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "database commit"):
            self.build()

    def test_permission_policy_mismatch_fails_closed(self) -> None:
        write_json(
            self.evidence / "workflow-trust.json",
            {
                "schema": 1,
                "ok": False,
                "workflows": [".github/workflows/ci.yml"],
                "action_metadata": ["action.yaml"],
                "findings": [{"code": "permission_mismatch"}],
            },
        )
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "not a successful schema-1 audit"):
            self.build()

    def test_mutable_proof_container_fails_closed(self) -> None:
        write_json(
            self.evidence / "workflow-trust.json",
            {
                "schema": 1,
                "ok": False,
                "workflows": [".github/workflows/ci.yml"],
                "action_metadata": ["action.yaml"],
                "findings": [{"code": "mutable_container", "path": ".github/workflows/ci.yml"}],
            },
        )
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "not a successful schema-1 audit"):
            self.build()

    def test_missing_action_yaml_coverage_fails_closed(self) -> None:
        write_json(
            self.evidence / "workflow-trust.json",
            {
                "schema": 1,
                "ok": True,
                "workflows": [".github/workflows/ci.yml"],
                "action_metadata": [],
                "findings": [],
            },
        )
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "does not cover both exact Action metadata forms"):
            self.build()

    def test_dirty_or_unexpected_source_fails_closed(self) -> None:
        (self.root / "unexpected.txt").write_text("dirty\n", encoding="utf-8")
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "dirty or has unexpected files"):
            self.build()


if __name__ == "__main__":
    unittest.main()
