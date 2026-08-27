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
            "Cargo.lock": "# fixture lock\n",
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
        write_json(
            self.evidence / "dependency-inventory.json",
            {
                "schema": 2,
                "ok": True,
                "package_count": 0,
                "packages": [],
                "unknown_license": [],
            },
        )
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
            {"vulnerabilities": {"found": False, "count": 0, "list": []}},
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
        (self.evidence / "cargo-deny-proof.json").unlink()
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "required evidence is missing"):
            self.build()

    def test_malformed_evidence_fails_closed(self) -> None:
        (self.evidence / "dependency-inventory.json").write_text("{", encoding="utf-8")
        with self.assertRaisesRegex(SUMMARY.AssuranceError, "invalid JSON evidence"):
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
