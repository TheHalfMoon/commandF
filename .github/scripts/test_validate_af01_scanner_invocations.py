#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
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


if __name__ == "__main__":
    unittest.main()
