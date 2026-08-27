#!/usr/bin/env python3
from __future__ import annotations

import copy
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("audit_workflow_trust.py")
SPEC = importlib.util.spec_from_file_location("audit_workflow_trust", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
AUDIT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = AUDIT
SPEC.loader.exec_module(AUDIT)

WORKFLOW = ".github/workflows/example.yml"
ACTION_YAML = "tools/example/action.yaml"
CHECKOUT_SHA = "fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09"
RUST_SHA = "032958afbdc797a9164d3bc0b56325c1308924a5"
CONTAINER_DIGEST = "9" * 64


def policy() -> dict:
    rules = {
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
    }
    return {
        "schema": 1,
        "rules": rules,
        "rationales": {
            key: f"Test rationale for {key} that is deliberately substantive."
            for key in rules
        },
        "workflows": {
            WORKFLOW: {
                "jobs": {
                    "build": {
                        "permissions": {"contents": "read"},
                        "runner": "ubuntu-24.04",
                        "timeout_minutes": 10,
                    }
                }
            }
        },
        "exceptions": [],
    }


def valid_workflow(extra_steps: str = "") -> str:
    return f"""name: example
on:
  pull_request:
permissions:
  contents: read
jobs:
  build:
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    container:
      image: docker.io/library/rust@sha256:{CONTAINER_DIGEST}
    steps:
      - uses: actions/checkout@{CHECKOUT_SHA}
        with:
          persist-credentials: false
      - uses: dtolnay/rust-toolchain@{RUST_SHA}
      - name: Test
        run: cargo test --locked --workspace
{extra_steps}"""


def valid_action() -> str:
    return """name: example action
description: fixture
runs:
  using: composite
  steps:
    - shell: bash
      run: echo ok
"""


class WorkflowTrustAuditTests(unittest.TestCase):
    def run_repo(
        self,
        workflow: str | None = None,
        action: str | None = None,
        audit_policy: dict | None = None,
        tracked: list[str] | None = None,
    ) -> dict:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths: list[str] = []
            if workflow is not None:
                path = root / WORKFLOW
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(workflow, encoding="utf-8")
                paths.append(WORKFLOW)
            if action is not None:
                path = root / ACTION_YAML
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(action, encoding="utf-8")
                paths.append(ACTION_YAML)
            return AUDIT.audit_repository(
                root,
                copy.deepcopy(audit_policy if audit_policy is not None else policy()),
                tracked_files=tracked if tracked is not None else paths,
            )

    @staticmethod
    def codes(result: dict) -> list[str]:
        return [finding["code"] for finding in result["findings"]]

    def test_valid_fixture_passes_and_is_deterministic(self) -> None:
        first = self.run_repo(valid_workflow(), valid_action())
        second = self.run_repo(valid_workflow(), valid_action())
        self.assertTrue(first["ok"], first)
        self.assertEqual(first, second)
        self.assertEqual(first["workflows"], [WORKFLOW])
        self.assertEqual(first["action_metadata"], [ACTION_YAML])

    def test_discovers_both_action_metadata_filenames_anywhere(self) -> None:
        workflows, actions = AUDIT.discover_security_files(
            [
                ".github/workflows/a.yml",
                ".github/workflows/b.yaml",
                "action.yml",
                "nested/one/action.yaml",
                "nested/two/not-action.yml",
            ]
        )
        self.assertEqual(workflows, [".github/workflows/a.yml", ".github/workflows/b.yaml"])
        self.assertEqual(actions, ["action.yml", "nested/one/action.yaml"])

    def test_new_workflow_not_in_policy_fails_closed(self) -> None:
        result = self.run_repo(
            valid_workflow(),
            tracked=[WORKFLOW, ".github/workflows/unplanned.yaml"],
        )
        self.assertIn("unplanned_workflow", self.codes(result))
        self.assertFalse(result["ok"])

    def test_mutable_external_action_tag_is_rejected(self) -> None:
        workflow = valid_workflow().replace(
            f"actions/checkout@{CHECKOUT_SHA}", "actions/checkout@v5"
        )
        result = self.run_repo(workflow)
        self.assertIn("mutable_uses", self.codes(result))

    def test_short_sha_and_branch_refs_are_rejected(self) -> None:
        for reference in ("owner/action@abc1234", "owner/action@main"):
            with self.subTest(reference=reference):
                workflow = valid_workflow(f"      - uses: {reference}\n")
                result = self.run_repo(workflow)
                self.assertIn("mutable_uses", self.codes(result))

    def test_mutable_reusable_workflow_reference_is_rejected(self) -> None:
        workflow = valid_workflow(
            "      - uses: owner/repository/.github/workflows/reuse.yml@main\n"
        )
        result = self.run_repo(workflow)
        self.assertIn("mutable_uses", self.codes(result))

    def test_mutable_external_uses_in_nested_action_yaml_is_rejected(self) -> None:
        action = """name: nested
description: fixture
runs:
  using: composite
  steps:
    - uses: owner/action@v1
"""
        result = self.run_repo(valid_workflow(), action)
        self.assertIn("mutable_uses", self.codes(result))

    def test_flow_style_workflow_uses_fails_closed(self) -> None:
        workflow = valid_workflow().replace(
            "    steps:\n      - uses:", "    steps: [{uses: owner/action@v1}]\n    ignored:\n      - uses:"
        )
        result = self.run_repo(workflow)
        self.assertIn("unsupported_trust_syntax", self.codes(result))

    def test_flow_style_action_metadata_uses_fails_closed(self) -> None:
        action = """name: nested
description: fixture
runs:
  using: composite
  steps: [{uses: owner/action@v1}]
"""
        result = self.run_repo(valid_workflow(), action)
        self.assertIn("unsupported_trust_syntax", self.codes(result))

    def test_checkout_credentials_must_be_disabled(self) -> None:
        workflow = valid_workflow().replace(
            "        with:\n          persist-credentials: false\n", ""
        )
        result = self.run_repo(workflow)
        self.assertIn("checkout_credentials", self.codes(result))

    def test_unscoped_persist_credentials_key_does_not_satisfy_checkout(self) -> None:
        workflow = valid_workflow().replace(
            "        with:\n          persist-credentials: false",
            "        env:\n          persist-credentials: false",
        )
        result = self.run_repo(workflow)
        self.assertIn("checkout_credentials", self.codes(result))

    def test_named_checkout_step_with_scoped_input_passes(self) -> None:
        workflow = valid_workflow().replace(
            f"      - uses: actions/checkout@{CHECKOUT_SHA}",
            f"      - name: Checkout\n        uses: actions/checkout@{CHECKOUT_SHA}",
        )
        result = self.run_repo(workflow)
        self.assertTrue(result["ok"], result)

    def test_action_metadata_checkout_credentials_must_be_disabled(self) -> None:
        action = f"""name: nested
description: fixture
runs:
  using: composite
  steps:
    - uses: actions/checkout@{CHECKOUT_SHA}
"""
        result = self.run_repo(valid_workflow(), action)
        self.assertIn("checkout_credentials", self.codes(result))

    def test_action_metadata_checkout_with_credentials_disabled_passes(self) -> None:
        action = f"""name: nested
description: fixture
runs:
  using: composite
  steps:
    - uses: actions/checkout@{CHECKOUT_SHA}
      with:
        persist-credentials: false
"""
        result = self.run_repo(valid_workflow(), action)
        self.assertTrue(result["ok"], result)

    def test_unresolved_default_permissions_fail_closed(self) -> None:
        workflow = valid_workflow().replace("permissions:\n  contents: read\n", "")
        result = self.run_repo(workflow)
        self.assertIn("unresolved_permissions", self.codes(result))

    def test_overbroad_permission_is_rejected(self) -> None:
        workflow = valid_workflow().replace(
            "permissions:\n  contents: read\n",
            "permissions:\n  contents: write\n",
        )
        result = self.run_repo(workflow)
        self.assertIn("permission_mismatch", self.codes(result))

    def test_policy_cannot_authorize_write_permission_in_stack_a(self) -> None:
        broken = policy()
        broken["workflows"][WORKFLOW]["jobs"]["build"]["permissions"] = {
            "contents": "write"
        }
        result = self.run_repo(valid_workflow(), audit_policy=broken)
        self.assertIn("invalid_policy", self.codes(result))

    def test_mutable_runner_is_rejected(self) -> None:
        workflow = valid_workflow().replace("ubuntu-24.04", "ubuntu-latest", 1)
        result = self.run_repo(workflow)
        self.assertIn("mutable_runner", self.codes(result))

    def test_missing_or_excessive_timeout_is_rejected(self) -> None:
        for workflow in (
            valid_workflow().replace("    timeout-minutes: 10\n", ""),
            valid_workflow().replace("timeout-minutes: 10", "timeout-minutes: 11"),
        ):
            with self.subTest():
                result = self.run_repo(workflow)
                self.assertIn("timeout_policy", self.codes(result))

    def test_mutable_job_and_service_container_images_are_rejected(self) -> None:
        mutable_job = valid_workflow().replace(
            f"docker.io/library/rust@sha256:{CONTAINER_DIGEST}", "rust:1.97.1"
        )
        result = self.run_repo(mutable_job)
        self.assertIn("mutable_container", self.codes(result))

        mutable_service = valid_workflow().replace(
            "    steps:\n",
            "    services:\n      database:\n        image: postgres:18\n    steps:\n",
        )
        result = self.run_repo(mutable_service)
        self.assertIn("mutable_container", self.codes(result))

    def test_non_container_image_key_does_not_false_positive(self) -> None:
        workflow = valid_workflow().replace(
            "    steps:\n",
            "    env:\n      image: mutable-but-not-a-container-authority\n    steps:\n",
        )
        result = self.run_repo(workflow)
        self.assertTrue(result["ok"], result)

    def test_unlocked_cargo_command_is_rejected(self) -> None:
        workflow = valid_workflow().replace(
            "cargo test --locked --workspace", "cargo test --workspace"
        )
        result = self.run_repo(workflow)
        self.assertIn("cargo_unlocked", self.codes(result))

    def test_comment_cannot_fake_cargo_locked_flag(self) -> None:
        workflow = valid_workflow().replace(
            "cargo test --locked --workspace", "cargo test --workspace # --locked"
        )
        result = self.run_repo(workflow)
        self.assertIn("cargo_unlocked", self.codes(result))

    def test_multiline_locked_cargo_command_is_accepted(self) -> None:
        continuation = chr(92)
        replacement = (
            "        run: |\n"
            f"          cargo test {continuation}\n"
            "            --locked --workspace"
        )
        workflow = valid_workflow().replace(
            "        run: cargo test --locked --workspace", replacement
        )
        result = self.run_repo(workflow)
        self.assertTrue(result["ok"], result)

    def test_cargo_and_subcommand_line_continuation_is_accepted(self) -> None:
        continuation = chr(92)
        replacement = (
            "        run: |\n"
            f"          cargo {continuation}\n"
            "            test --locked --workspace"
        )
        workflow = valid_workflow().replace(
            "        run: cargo test --locked --workspace", replacement
        )
        result = self.run_repo(workflow)
        self.assertTrue(result["ok"], result)

    def test_later_locked_cargo_command_cannot_mask_unlocked_command(self) -> None:
        workflow = valid_workflow().replace(
            "cargo test --locked --workspace",
            "cargo test --workspace && cargo test --locked -p commandf-pkg",
        )
        result = self.run_repo(workflow)
        cargo_findings = [
            finding for finding in result["findings"] if finding["code"] == "cargo_unlocked"
        ]
        self.assertEqual(len(cargo_findings), 1, result)
        self.assertIn("cargo test --workspace", cargo_findings[0]["detail"])

    def test_cargo_global_option_syntax_fails_closed(self) -> None:
        workflow = valid_workflow().replace(
            "cargo test --locked --workspace",
            "cargo --color always test --locked --workspace",
        )
        result = self.run_repo(workflow)
        self.assertIn("unsupported_cargo_syntax", self.codes(result))

    def test_malformed_workflow_fails_closed(self) -> None:
        result = self.run_repo("name: broken\n\tjobs:\n")
        self.assertIn("malformed_yaml", self.codes(result))
        self.assertFalse(result["ok"])

    def test_missing_jobs_mapping_fails_closed(self) -> None:
        result = self.run_repo("name: broken\non:\n  pull_request:\n")
        self.assertIn("malformed_yaml", self.codes(result))

    def test_exception_requires_reason_and_revisit(self) -> None:
        broken = policy()
        broken["exceptions"] = [
            {"rule": "mutable_runner", "path": WORKFLOW, "reason": "short"}
        ]
        result = self.run_repo(valid_workflow(), audit_policy=broken)
        self.assertIn("invalid_policy", self.codes(result))

    def test_non_object_exception_fails_closed(self) -> None:
        broken = policy()
        broken["exceptions"] = [None]
        result = self.run_repo(valid_workflow(), audit_policy=broken)
        self.assertFalse(result["ok"])
        self.assertIn("invalid_policy", self.codes(result))

    def test_malformed_rule_types_fail_closed(self) -> None:
        for key, value in (
            ("cargo_locked_subcommands", "test"),
            ("require_external_uses_full_sha", "true"),
            ("require_container_digest", False),
        ):
            with self.subTest(key=key, value=value):
                broken = policy()
                broken["rules"][key] = value
                result = self.run_repo(valid_workflow(), audit_policy=broken)
                self.assertFalse(result["ok"])
                self.assertIn("invalid_policy", self.codes(result))

    def test_missing_rule_rationale_fails_closed(self) -> None:
        broken = policy()
        del broken["rationales"]["require_container_digest"]
        result = self.run_repo(valid_workflow(), audit_policy=broken)
        self.assertIn("invalid_policy", self.codes(result))

    def test_invalid_policy_root_fails_closed_without_exception(self) -> None:
        result = self.run_repo(valid_workflow(), audit_policy={"schema": 1})
        self.assertFalse(result["ok"])
        self.assertIn("invalid_policy", self.codes(result))
        self.assertEqual(result["workflows"], [])
        self.assertEqual(result["action_metadata"], [])


if __name__ == "__main__":
    unittest.main()
