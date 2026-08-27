#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("audit_workflow_trust_surface.py")
SPEC = importlib.util.spec_from_file_location("audit_workflow_trust_surface_precision_target", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
SURFACE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = SURFACE
SPEC.loader.exec_module(SURFACE)

LOCKED = {"bench", "build", "check", "clippy", "doc", "metadata", "run", "test"}


def codes(findings: list[object]) -> list[str]:
    return [finding.code for finding in findings]


class SurfacePrecisionTests(unittest.TestCase):
    def test_diagnostic_text_containing_cargo_is_not_executable_authority(self) -> None:
        findings = SURFACE._direct_cargo_findings(
            "scripts/github-action.sh",
            "action-script",
            'emit_operational_failure "rustup and cargo are required to build the source-backed Action"',
            LOCKED,
        )
        self.assertNotIn("unsupported_cargo_indirect", codes(findings))

    def test_toolchain_selected_cargo_version_with_fixed_redirections_is_allowed(self) -> None:
        findings = SURFACE._direct_cargo_findings(
            "scripts/github-action.sh",
            "action-script",
            "if ! cargo +1.97.1 --version >/dev/null 2>&1",
            LOCKED,
        )
        self.assertNotIn("unsupported_cargo_syntax", codes(findings))
        self.assertNotIn("cargo_unlocked", codes(findings))

    def test_cargo_information_flag_with_non_redirection_tail_fails_closed(self) -> None:
        findings = SURFACE._direct_cargo_findings(
            "action.yml", "composite-action", "cargo --version test", LOCKED
        )
        self.assertIn("unsupported_cargo_syntax", codes(findings))

    def test_unresolved_env_wrapper_cannot_hide_cargo(self) -> None:
        findings = SURFACE._direct_cargo_findings(
            "action.yml",
            "composite-action",
            "env -u UNUSED cargo test --workspace",
            LOCKED,
        )
        self.assertIn("unsupported_cargo_indirect", codes(findings))


if __name__ == "__main__":
    unittest.main()
