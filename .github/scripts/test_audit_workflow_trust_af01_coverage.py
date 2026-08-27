#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("audit_workflow_trust.py")
SPEC = importlib.util.spec_from_file_location("audit_workflow_trust_af01_coverage_target", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
AUDIT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = AUDIT
SPEC.loader.exec_module(AUDIT)

ROOT = Path(__file__).resolve().parents[2]
SECURITY_WORKFLOW = ROOT / ".github" / "workflows" / "af01-security.yml"


class Af01SecurityCoverageTests(unittest.TestCase):
    @staticmethod
    def pull_request_children(text: str) -> list[str]:
        lines = text.splitlines()
        try:
            start = lines.index("  pull_request:")
        except ValueError as error:
            raise AssertionError("af01-security must have a pull_request trigger") from error

        children: list[str] = []
        for line in lines[start + 1 :]:
            if line and not line.startswith(" "):
                break
            if line.startswith("  ") and not line.startswith("    ") and line.strip():
                break
            if line.strip():
                children.append(line.strip())
        return children

    def test_stack_b_security_gate_is_universal_for_pull_requests(self) -> None:
        text = SECURITY_WORKFLOW.read_text(encoding="utf-8")
        children = self.pull_request_children(text)
        self.assertFalse(
            any(child.startswith(("paths:", "paths-ignore:")) for child in children),
            "AF-01 Stack B must remain universal so policy/config/action metadata changes cannot bypass it",
        )

        covered_surfaces = (
            "deny.toml",
            ".github/workflow-trust-policy.json",
            ".github/workflows/af01-security.yml",
            ".github/scripts/audit_workflow_trust.py",
            "action.yml",
            "nested/action.yaml",
        )
        for surface in covered_surfaces:
            with self.subTest(surface=surface):
                self.assertEqual(children, [], f"universal pull_request coverage must include {surface}")

    def test_action_metadata_discovery_covers_both_supported_filenames(self) -> None:
        workflows, actions = AUDIT.discover_security_files(
            [
                ".github/workflows/af01-security.yml",
                "action.yml",
                "nested/action.yaml",
                "nested/not-an-action.yml",
            ]
        )
        self.assertEqual(workflows, [".github/workflows/af01-security.yml"])
        self.assertEqual(actions, ["action.yml", "nested/action.yaml"])


if __name__ == "__main__":
    unittest.main()
