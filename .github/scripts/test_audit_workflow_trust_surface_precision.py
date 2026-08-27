#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
import tempfile
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


def action(run: str) -> str:
    return f"""name: fixture
description: fixture
runs:
  using: composite
  steps:
    - shell: bash
      run: {run}
"""


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

    def _audit_env_delegation(self, delegated_script: str) -> dict:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "action.yml").write_text(
                action('env -u UNUSED bash "$GITHUB_ACTION_PATH/scripts/build.sh"'),
                encoding="utf-8",
            )
            (root / "scripts" / "build.sh").write_text(delegated_script, encoding="utf-8")
            return SURFACE.audit_repository_surface(
                root,
                {"rules": {"cargo_locked_subcommands": sorted(LOCKED)}},
                tracked_files=["action.yml", "scripts/build.sh"],
            )

    def test_env_option_operand_wrapper_rejects_locked_action_delegation(self) -> None:
        result = self._audit_env_delegation(
            "#!/usr/bin/env bash\ncargo test --locked --workspace\n"
        )
        self.assertFalse(result["ok"])
        self.assertIn("unsupported_action_script", [item["code"] for item in result["findings"]])

    def test_env_option_operand_wrapper_rejects_unlocked_action_delegation(self) -> None:
        result = self._audit_env_delegation("#!/usr/bin/env bash\ncargo test --workspace\n")
        self.assertFalse(result["ok"])
        self.assertIn("unsupported_action_script", [item["code"] for item in result["findings"]])

    def test_dot_slash_direct_script_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml", action("./scripts/build.sh"), LOCKED
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_bare_relative_direct_script_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml", action("scripts/build.sh"), LOCKED
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_workspace_direct_script_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml", action('"$GITHUB_WORKSPACE/scripts/build.sh"'), LOCKED
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def _audit_path_resolved_delegation(self, delegated_script: str) -> dict:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "action.yml").write_text(
                """name: fixture
description: fixture
runs:
  using: composite
  steps:
    - shell: bash
      run: |
        PATH="$GITHUB_ACTION_PATH/scripts:$PATH"
        build.sh
""",
                encoding="utf-8",
            )
            (root / "scripts" / "build.sh").write_text(delegated_script, encoding="utf-8")
            return SURFACE.audit_repository_surface(
                root,
                {"rules": {"cargo_locked_subcommands": sorted(LOCKED)}},
                tracked_files=["action.yml", "scripts/build.sh"],
            )

    def test_path_resolved_locked_action_script_fails_closed(self) -> None:
        result = self._audit_path_resolved_delegation(
            "#!/usr/bin/env bash\ncargo test --locked --workspace\n"
        )
        self.assertFalse(result["ok"])
        self.assertIn("unsupported_action_script", [item["code"] for item in result["findings"]])

    def test_path_resolved_unlocked_action_script_fails_closed(self) -> None:
        result = self._audit_path_resolved_delegation(
            "#!/usr/bin/env bash\ncargo test --workspace\n"
        )
        self.assertFalse(result["ok"])
        self.assertIn("unsupported_action_script", [item["code"] for item in result["findings"]])

    def test_scoped_path_assignment_before_bare_command_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action('PATH="$GITHUB_ACTION_PATH/scripts:$PATH" build.sh'),
            LOCKED,
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_env_path_assignment_before_bare_command_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action('env PATH="$GITHUB_ACTION_PATH/scripts:$PATH" build.sh'),
            LOCKED,
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_path_assignment_text_as_argument_is_not_path_mutation(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action("printf 'PATH=/tmp/example\\n'"),
            LOCKED,
        )
        self.assertNotIn("unsupported_action_script", codes(findings))

    def _audit_builtin_path_writer(self, writer: str, delegated_script: str) -> dict:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "action.yml").write_text(
                f"""name: fixture
description: fixture
runs:
  using: composite
  steps:
    - shell: bash
      run: |
        {writer}
        build.sh
""",
                encoding="utf-8",
            )
            (root / "scripts" / "build.sh").write_text(delegated_script, encoding="utf-8")
            return SURFACE.audit_repository_surface(
                root,
                {"rules": {"cargo_locked_subcommands": sorted(LOCKED)}},
                tracked_files=["action.yml", "scripts/build.sh"],
            )

    def test_read_path_writer_rejects_locked_bare_action_script(self) -> None:
        result = self._audit_builtin_path_writer(
            'read -r PATH <<< "$GITHUB_ACTION_PATH/scripts:$PATH"',
            "#!/usr/bin/env bash\ncargo test --locked --workspace\n",
        )
        self.assertFalse(result["ok"])
        self.assertIn("unsupported_action_script", [item["code"] for item in result["findings"]])

    def test_read_path_writer_rejects_unlocked_bare_action_script(self) -> None:
        result = self._audit_builtin_path_writer(
            'read -r PATH <<< "$GITHUB_ACTION_PATH/scripts:$PATH"',
            "#!/usr/bin/env bash\ncargo test --workspace\n",
        )
        self.assertFalse(result["ok"])
        self.assertIn("unsupported_action_script", [item["code"] for item in result["findings"]])

    def test_printf_v_path_writer_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action("printf -v PATH '%s' /tmp/bin"),
            LOCKED,
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_mapfile_path_writer_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action("mapfile PATH </tmp/paths"),
            LOCKED,
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_readarray_path_writer_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action("readarray PATH </tmp/paths"),
            LOCKED,
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_getopts_path_writer_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action("getopts ab PATH"),
            LOCKED,
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_nameref_builtin_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action("declare -n SEARCH_PATH=PATH"),
            LOCKED,
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_read_non_path_target_is_allowed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action("read -r VALUE </tmp/value"),
            LOCKED,
        )
        self.assertNotIn("unsupported_action_script", codes(findings))

    def test_printf_v_non_path_target_is_allowed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action("printf -v VALUE '%s' ok"),
            LOCKED,
        )
        self.assertNotIn("unsupported_action_script", codes(findings))


if __name__ == "__main__":
    unittest.main()
