#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("audit_workflow_trust.py")
SPEC = importlib.util.spec_from_file_location("audit_workflow_trust_syntax_target", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
AUDIT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = AUDIT
SPEC.loader.exec_module(AUDIT)

CHECKOUT_SHA = "fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09"
CONTAINER_DIGEST = "9" * 64
PATH = ".github/workflows/example.yml"
EXPECTED = {
    "jobs": {
        "build": {
            "permissions": {"contents": "read"},
            "runner": "ubuntu-24.04",
            "timeout_minutes": 10,
        }
    }
}
POLICY = {
    "rules": {
        "cargo_locked_subcommands": [
            "bench",
            "build",
            "check",
            "clippy",
            "doc",
            "metadata",
            "run",
            "test",
        ],
        "require_container_digest": True,
        "require_checkout_credentials_disabled": True,
        "require_external_uses_full_sha": True,
    },
    "exceptions": [],
}


def workflow(step: str, container: str | None = None) -> str:
    container_block = container or (
        "    container:\n"
        f"      image: docker.io/library/rust@sha256:{CONTAINER_DIGEST}\n"
    )
    return f"""name: example
on:
  pull_request:
permissions:
  contents: read
jobs:
  build:
    runs-on: ubuntu-24.04
    timeout-minutes: 10
{container_block}    steps:
{step}
"""


class QuotedTrustSyntaxTests(unittest.TestCase):
    @staticmethod
    def codes(findings: list[object]) -> list[str]:
        return [finding.code for finding in findings]

    def test_quoted_uses_in_workflow_fails_closed(self) -> None:
        text = workflow(
            f"      - \"uses\": actions/checkout@{CHECKOUT_SHA}\n"
            "        with:\n"
            "          persist-credentials: false"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_trust_syntax", self.codes(findings))

    def test_quoted_uses_in_action_metadata_fails_closed(self) -> None:
        text = f"""name: example
description: fixture
runs:
  using: composite
  steps:
    - 'uses': actions/checkout@{CHECKOUT_SHA}
      with:
        persist-credentials: false
"""
        findings = AUDIT.audit_action_metadata("nested/action.yaml", text, POLICY)
        self.assertIn("unsupported_trust_syntax", self.codes(findings))

    def test_quoted_job_container_key_fails_closed(self) -> None:
        text = workflow(
            "      - name: Test\n"
            "        run: cargo test --locked --workspace",
            container=(
                "    \"container\":\n"
                f"      image: docker.io/library/rust@sha256:{CONTAINER_DIGEST}\n"
            ),
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_trust_syntax", self.codes(findings))

    def test_quoted_container_image_key_fails_closed(self) -> None:
        text = workflow(
            "      - name: Test\n"
            "        run: cargo test --locked --workspace",
            container=(
                "    container:\n"
                f"      \"image\": docker.io/library/rust@sha256:{CONTAINER_DIGEST}\n"
            ),
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_trust_syntax", self.codes(findings))

    def test_flow_style_services_fails_closed(self) -> None:
        text = workflow(
            "      - name: Test\n"
            "        run: cargo test --locked --workspace"
        ).replace(
            "    steps:\n",
            "    services: {db: {image: postgres:18}}\n    steps:\n",
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_trust_syntax", self.codes(findings))

    def test_quoted_job_permission_override_fails_closed(self) -> None:
        text = workflow(
            "      - name: Test\n"
            "        run: cargo test --locked --workspace"
        ).replace(
            "    steps:\n",
            "    \"permissions\":\n      contents: write\n    steps:\n",
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_trust_syntax", self.codes(findings))

    def test_quoted_image_under_env_is_not_container_authority(self) -> None:
        text = workflow(
            "      - name: Test\n"
            "        env:\n"
            "          \"image\": harmless-string\n"
            "        run: cargo test --locked --workspace"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertNotIn("unsupported_trust_syntax", self.codes(findings))

    def test_quoted_uses_text_inside_run_block_is_ignored(self) -> None:
        text = workflow(
            "      - name: Test\n"
            "        run: |\n"
            "          printf '%s\\n' '- \"uses\": owner/action@v1'\n"
            "          cargo test --locked --workspace"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertNotIn("unsupported_trust_syntax", self.codes(findings))

    def test_cargo_version_is_non_lockfile_introspection(self) -> None:
        text = workflow(
            "      - name: Version\n"
            "        run: cargo --version"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        codes = self.codes(findings)
        self.assertNotIn("unsupported_cargo_syntax", codes)
        self.assertNotIn("cargo_unlocked", codes)

    def test_toolchain_selected_cargo_version_is_non_lockfile_introspection(self) -> None:
        text = workflow(
            "      - name: Version\n"
            "        run: cargo +1.97.1 --version"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        codes = self.codes(findings)
        self.assertNotIn("unsupported_cargo_syntax", codes)
        self.assertNotIn("cargo_unlocked", codes)

    def test_cargo_version_with_extra_tokens_fails_closed(self) -> None:
        text = workflow(
            "      - name: Invalid\n"
            "        run: cargo --version test"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_cargo_syntax", self.codes(findings))

    def test_variable_expanded_cargo_command_fails_closed(self) -> None:
        text = workflow(
            "      - name: Indirect\n"
            "        run: |\n"
            "          tool=cargo\n"
            "          \"$tool\" test --workspace"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_cargo_indirect", self.codes(findings))

    def test_command_substitution_cargo_command_fails_closed(self) -> None:
        text = workflow(
            "      - name: Indirect\n"
            "        run: $(command -v cargo) test --workspace"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_cargo_indirect", self.codes(findings))

    def test_eval_cargo_command_fails_closed(self) -> None:
        text = workflow(
            "      - name: Indirect\n"
            "        run: eval 'cargo test --workspace'"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_cargo_indirect", self.codes(findings))

    def test_nested_shell_cargo_command_fails_closed(self) -> None:
        text = workflow(
            "      - name: Indirect\n"
            "        run: bash -c 'cargo test --workspace'"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertIn("unsupported_cargo_indirect", self.codes(findings))

    def test_dynamic_path_executable_does_not_false_positive(self) -> None:
        text = workflow(
            "      - name: Java\n"
            "        run: \"$JAVA_HOME/bin/java\" -version"
        )
        findings = AUDIT.audit_workflow(PATH, text, EXPECTED, POLICY)
        self.assertNotIn("unsupported_cargo_indirect", self.codes(findings))


if __name__ == "__main__":
    unittest.main()
