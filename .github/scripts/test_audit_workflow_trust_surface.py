#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("audit_workflow_trust_surface.py")
SPEC = importlib.util.spec_from_file_location("audit_workflow_trust_surface_target", MODULE_PATH)
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


class ShellAuthoritySurfaceTests(unittest.TestCase):
    def test_bash_heredoc_is_executable_authority(self) -> None:
        script = "bash <<'SCRIPT'\ncargo test --workspace\nSCRIPT"
        findings = SURFACE._shell_heredoc_findings("wf.yml", "build", script)
        self.assertIn("unsupported_shell_heredoc", codes(findings))

    def test_sh_heredoc_is_executable_authority(self) -> None:
        script = "sh <<EOF\ncargo test --workspace\nEOF"
        findings = SURFACE._shell_heredoc_findings("wf.yml", "build", script)
        self.assertIn("unsupported_shell_heredoc", codes(findings))

    def test_python_heredoc_remains_data(self) -> None:
        script = "python3 - <<'PY'\nprint('cargo test --workspace')\nPY"
        findings = SURFACE._shell_heredoc_findings("wf.yml", "build", script)
        self.assertNotIn("unsupported_shell_heredoc", codes(findings))

    def test_double_bracket_boolean_expression_is_not_executable(self) -> None:
        script = 'if [[ "$code" == "0" || "$code" == "2" ]]; then\n  echo ok\nfi'
        findings = SURFACE._direct_cargo_findings("script.sh", "action-script", script, LOCKED)
        self.assertNotIn("unsupported_cargo_indirect", codes(findings))

    def test_double_bracket_command_substitution_fails_closed(self) -> None:
        script = 'if [[ "$(printf test)" == "test" ]]; then\n  echo ok\nfi'
        findings = SURFACE._direct_cargo_findings("script.sh", "action-script", script, LOCKED)
        self.assertIn("unsupported_cargo_indirect", codes(findings))

    def test_action_yml_unlocked_cargo_fails(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml", action("cargo test --workspace"), LOCKED
        )
        self.assertIn("cargo_unlocked", codes(findings))

    def test_action_yaml_unlocked_cargo_fails(self) -> None:
        findings = SURFACE.audit_action_text(
            "nested/action.yaml", action("cargo build --workspace"), LOCKED
        )
        self.assertIn("cargo_unlocked", codes(findings))

    def test_action_yml_locked_cargo_passes(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml", action("cargo test --locked --workspace"), LOCKED
        )
        self.assertNotIn("cargo_unlocked", codes(findings))

    def test_action_yaml_locked_cargo_passes(self) -> None:
        findings = SURFACE.audit_action_text(
            "nested/action.yaml", action("cargo build --locked --workspace"), LOCKED
        )
        self.assertNotIn("cargo_unlocked", codes(findings))

    def test_exported_cargo_variable_executable_fails_closed(self) -> None:
        script = 'export tool=cargo\n"$tool" test --workspace'
        findings = SURFACE._direct_cargo_findings("action.yml", "composite-action", script, LOCKED)
        self.assertIn("unsupported_cargo_indirect", codes(findings))

    def test_readonly_cargo_variable_executable_fails_closed(self) -> None:
        script = 'readonly tool=/usr/bin/cargo\n"$tool" build --workspace'
        findings = SURFACE._direct_cargo_findings("action.yml", "composite-action", script, LOCKED)
        self.assertIn("unsupported_cargo_indirect", codes(findings))

    def test_unknown_variable_executable_fails_closed(self) -> None:
        findings = SURFACE._direct_cargo_findings(
            "action.yml", "composite-action", '"$tool" test --workspace', LOCKED
        )
        self.assertIn("unsupported_cargo_indirect", codes(findings))

    def test_fixed_non_cargo_variable_executable_is_proven_safe(self) -> None:
        script = 'binary="$CARGO_TARGET_DIR/debug/commandf"\n"$binary" check fixture'
        findings = SURFACE._direct_cargo_findings("script.sh", "action-script", script, LOCKED)
        self.assertNotIn("unsupported_cargo_indirect", codes(findings))

    def test_action_dynamic_shell_source_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml", action('bash "$SCRIPT"'), LOCKED
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_action_shell_c_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml", action("bash -c 'cargo test --locked --workspace'"), LOCKED
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_action_relative_shell_source_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml", action("bash scripts/build.sh"), LOCKED
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_action_root_script_suffix_expansion_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action('bash "$GITHUB_ACTION_PATH/scripts/build.sh$SUFFIX"'),
            LOCKED,
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_action_root_script_prefix_expansion_fails_closed(self) -> None:
        findings = SURFACE.audit_action_text(
            "action.yml",
            action('bash "$PREFIX$GITHUB_ACTION_PATH/scripts/build.sh"'),
            LOCKED,
        )
        self.assertIn("unsupported_action_script", codes(findings))

    def test_tracked_local_action_script_is_audited(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "action.yml").write_text(
                action('bash "$GITHUB_ACTION_PATH/scripts/build.sh"'), encoding="utf-8"
            )
            (root / "scripts" / "build.sh").write_text(
                "#!/usr/bin/env bash\ncargo test --workspace\n", encoding="utf-8"
            )
            result = SURFACE.audit_repository_surface(
                root,
                {"rules": {"cargo_locked_subcommands": sorted(LOCKED)}},
                tracked_files=["action.yml", "scripts/build.sh"],
            )
            self.assertFalse(result["ok"])
            self.assertIn("cargo_unlocked", [item["code"] for item in result["findings"]])

    def test_tracked_local_locked_action_script_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "action.yml").write_text(
                action('bash "$GITHUB_ACTION_PATH/scripts/build.sh"'), encoding="utf-8"
            )
            (root / "scripts" / "build.sh").write_text(
                "#!/usr/bin/env bash\ncargo test --locked --workspace\n", encoding="utf-8"
            )
            result = SURFACE.audit_repository_surface(
                root,
                {"rules": {"cargo_locked_subcommands": sorted(LOCKED)}},
                tracked_files=["action.yml", "scripts/build.sh"],
            )
            self.assertTrue(result["ok"], result["findings"])

    def test_nested_tracked_local_action_script_is_audited(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "action.yml").write_text(
                action('bash "$GITHUB_ACTION_PATH/scripts/entry.sh"'), encoding="utf-8"
            )
            (root / "scripts" / "entry.sh").write_text(
                '#!/usr/bin/env bash\nexec bash "$GITHUB_ACTION_PATH/scripts/build.sh"\n',
                encoding="utf-8",
            )
            (root / "scripts" / "build.sh").write_text(
                "#!/usr/bin/env bash\ncargo test --workspace\n", encoding="utf-8"
            )
            result = SURFACE.audit_repository_surface(
                root,
                {"rules": {"cargo_locked_subcommands": sorted(LOCKED)}},
                tracked_files=["action.yml", "scripts/entry.sh", "scripts/build.sh"],
            )
            self.assertFalse(result["ok"])
            self.assertIn("cargo_unlocked", [item["code"] for item in result["findings"]])

    def test_nested_dynamic_local_action_script_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "action.yml").write_text(
                action('bash "$GITHUB_ACTION_PATH/scripts/entry.sh"'), encoding="utf-8"
            )
            (root / "scripts" / "entry.sh").write_text(
                '#!/usr/bin/env bash\nexec bash "$NEXT_SCRIPT"\n', encoding="utf-8"
            )
            result = SURFACE.audit_repository_surface(
                root,
                {"rules": {"cargo_locked_subcommands": sorted(LOCKED)}},
                tracked_files=["action.yml", "scripts/entry.sh"],
            )
            self.assertFalse(result["ok"])
            self.assertIn(
                "unsupported_action_script", [item["code"] for item in result["findings"]]
            )

    def test_recursive_action_script_cycle_is_bounded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "scripts").mkdir()
            (root / "action.yml").write_text(
                action('bash "$GITHUB_ACTION_PATH/scripts/a.sh"'), encoding="utf-8"
            )
            (root / "scripts" / "a.sh").write_text(
                '#!/usr/bin/env bash\nexec bash "$GITHUB_ACTION_PATH/scripts/b.sh"\n',
                encoding="utf-8",
            )
            (root / "scripts" / "b.sh").write_text(
                '#!/usr/bin/env bash\nexec bash "$GITHUB_ACTION_PATH/scripts/a.sh"\n',
                encoding="utf-8",
            )
            result = SURFACE.audit_repository_surface(
                root,
                {"rules": {"cargo_locked_subcommands": sorted(LOCKED)}},
                tracked_files=["action.yml", "scripts/a.sh", "scripts/b.sh"],
            )
            self.assertTrue(result["ok"], result["findings"])

    def test_live_repository_surface_is_clean(self) -> None:
        root = Path(__file__).resolve().parents[2]
        policy = json.loads(
            (root / ".github" / "workflow-trust-policy.json").read_text(encoding="utf-8")
        )
        result = SURFACE.audit_repository_surface(root, policy)
        self.assertTrue(result["ok"], result["findings"])


if __name__ == "__main__":
    unittest.main()
