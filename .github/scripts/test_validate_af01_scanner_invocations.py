#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("validate_af01_scanner_invocations.py")
SPEC = importlib.util.spec_from_file_location("af01_scanner_invocations", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
VALIDATE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = VALIDATE
SPEC.loader.exec_module(VALIDATE)

VERIFIED_PATH = Path(__file__).with_name("build_af01_assurance_summary_verified.py")
VERIFIED_SPEC = importlib.util.spec_from_file_location("af01_verified_scanner_binding", VERIFIED_PATH)
assert VERIFIED_SPEC is not None and VERIFIED_SPEC.loader is not None
VERIFIED = importlib.util.module_from_spec(VERIFIED_SPEC)
sys.modules[VERIFIED_SPEC.name] = VERIFIED
VERIFIED_SPEC.loader.exec_module(VERIFIED)


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def workflow_text() -> str:
    return f"""name: assurance
jobs:
  assurance-proof:
    runs-on: ubuntu-24.04
    steps:
      - name: cargo deny
        uses: {VALIDATE.CARGO_DENY_USES}
        with:
          command: check
          arguments: --all-features
          command-arguments: advisories bans licenses sources
          log-level: warn
      - name: install audit
        run: {VALIDATE.CARGO_AUDIT_INSTALL}
      - name: audit
        run: |
          set -euo pipefail
          {VALIDATE.CARGO_AUDIT_RUN}
      - name: zizmor
        uses: {VALIDATE.ZIZMOR_USES}
        with:
          inputs: .
          collect: all
          online-audits: false
          persona: regular
          min-severity: medium
          version: 1.29.0
          advanced-security: false
          color: false
          annotations: false
          fail-on-no-inputs: true
"""


class ScannerInvocationContractTests(unittest.TestCase):
    def validate(self, content: str) -> dict[str, object]:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "workflow.yml"
            path.write_text(content, encoding="utf-8")
            return VALIDATE.validate_workflow(path)

    def proof_fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path]:
        temp = tempfile.TemporaryDirectory()
        base = Path(temp.name)
        root = base / "repo"
        evidence = base / "evidence"
        workflow = root / VERIFIED.ASSURANCE_WORKFLOW
        workflow.parent.mkdir(parents=True, exist_ok=True)
        workflow.write_text(workflow_text(), encoding="utf-8")
        evidence.mkdir()
        write_json(
            evidence / VERIFIED.BASE.EVIDENCE_FILES["cargo_deny"],
            {
                "action_commit": VALIDATE.CARGO_DENY_USES.rsplit("@", 1)[1],
                "checks": ["advisories", "bans", "licenses", "sources"],
            },
        )
        write_json(
            evidence / VERIFIED.BASE.EVIDENCE_FILES["cargo_audit"],
            {"cargo_audit_version": "0.22.2"},
        )
        write_json(
            evidence / VERIFIED.BASE.EVIDENCE_FILES["zizmor"],
            {
                "action_commit": VALIDATE.ZIZMOR_USES.rsplit("@", 1)[1],
                "zizmor_version": "1.29.0",
                "min_severity": "medium",
                "online_audits": False,
                "advanced_security": False,
            },
        )
        return temp, root, evidence

    def test_exact_scanner_invocations_are_canonically_bound(self) -> None:
        result = self.validate(workflow_text())
        self.assertEqual(result["schema"], 1)
        self.assertEqual(result["cargo_deny"]["uses"], VALIDATE.CARGO_DENY_USES)
        self.assertEqual(result["cargo_deny"]["inputs"], VALIDATE.CARGO_DENY_INPUTS)
        self.assertEqual(result["zizmor"]["uses"], VALIDATE.ZIZMOR_USES)
        self.assertEqual(result["zizmor"]["inputs"], VALIDATE.ZIZMOR_INPUTS)
        self.assertEqual(result["cargo_audit"]["version"], "0.22.2")
        self.assertRegex(result["sha256"], r"^[0-9a-f]{64}$")

    def test_changed_cargo_deny_commit_fails_closed(self) -> None:
        changed = workflow_text().replace(
            VALIDATE.CARGO_DENY_USES,
            "EmbarkStudios/cargo-deny-action@" + "0" * 40,
        )
        with self.assertRaisesRegex(
            VALIDATE.ScannerContractError, "cargo-deny executed action commit"
        ):
            self.validate(changed)

    def test_changed_cargo_deny_arguments_fail_closed(self) -> None:
        changed = workflow_text().replace(
            "command-arguments: advisories bans licenses sources",
            "command-arguments: advisories",
        )
        with self.assertRaisesRegex(
            VALIDATE.ScannerContractError, "cargo-deny executed inputs"
        ):
            self.validate(changed)

    def test_changed_zizmor_version_fails_closed(self) -> None:
        changed = workflow_text().replace("version: 1.29.0", "version: 9.9.9")
        with self.assertRaisesRegex(
            VALIDATE.ScannerContractError, "zizmor executed inputs"
        ):
            self.validate(changed)

    def test_changed_cargo_audit_execution_fails_closed(self) -> None:
        changed = workflow_text().replace(
            VALIDATE.CARGO_AUDIT_RUN,
            "cargo audit --json",
        )
        with self.assertRaisesRegex(
            VALIDATE.ScannerContractError, "cargo-audit execution exact command"
        ):
            self.validate(changed)

    def test_cargo_deny_proof_must_match_executed_invocation(self) -> None:
        temp, root, evidence = self.proof_fixture()
        try:
            path = evidence / VERIFIED.BASE.EVIDENCE_FILES["cargo_deny"]
            proof = json.loads(path.read_text(encoding="utf-8"))
            proof["checks"] = ["advisories"]
            write_json(path, proof)
            with self.assertRaisesRegex(
                VERIFIED.BASE.AssuranceError,
                "cargo-deny proof does not match the executed action invocation",
            ):
                VERIFIED.validate_scanner_binding(root, evidence)
        finally:
            temp.cleanup()

    def test_zizmor_proof_must_match_executed_inputs(self) -> None:
        temp, root, evidence = self.proof_fixture()
        try:
            path = evidence / VERIFIED.BASE.EVIDENCE_FILES["zizmor"]
            proof = json.loads(path.read_text(encoding="utf-8"))
            proof["min_severity"] = "low"
            write_json(path, proof)
            with self.assertRaisesRegex(
                VERIFIED.BASE.AssuranceError,
                "zizmor proof does not match the executed action invocation",
            ):
                VERIFIED.validate_scanner_binding(root, evidence)
        finally:
            temp.cleanup()

    def test_cargo_audit_proof_must_match_executed_version(self) -> None:
        temp, root, evidence = self.proof_fixture()
        try:
            path = evidence / VERIFIED.BASE.EVIDENCE_FILES["cargo_audit"]
            proof = json.loads(path.read_text(encoding="utf-8"))
            proof["cargo_audit_version"] = "9.9.9"
            write_json(path, proof)
            with self.assertRaisesRegex(
                VERIFIED.BASE.AssuranceError,
                "cargo-audit proof does not match the executed install/run contract",
            ):
                VERIFIED.validate_scanner_binding(root, evidence)
        finally:
            temp.cleanup()

    def test_scanner_proofs_match_exact_executed_contract(self) -> None:
        temp, root, evidence = self.proof_fixture()
        try:
            contract = VERIFIED.validate_scanner_binding(root, evidence)
            self.assertEqual(contract["cargo_deny"]["uses"], VALIDATE.CARGO_DENY_USES)
            self.assertEqual(contract["zizmor"]["uses"], VALIDATE.ZIZMOR_USES)
            self.assertEqual(contract["cargo_audit"]["version"], "0.22.2")
        finally:
            temp.cleanup()


if __name__ == "__main__":
    unittest.main()
