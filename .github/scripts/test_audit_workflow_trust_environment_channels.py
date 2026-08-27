#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("audit_workflow_trust_environment_channels.py")
SPEC = importlib.util.spec_from_file_location("audit_workflow_trust_environment_channels_target", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
CHANNELS = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = CHANNELS
SPEC.loader.exec_module(CHANNELS)


def findings_for(files: dict[str, str]) -> list[dict[str, str]]:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for path, text in files.items():
            candidate = root / path
            candidate.parent.mkdir(parents=True, exist_ok=True)
            candidate.write_text(text, encoding="utf-8")
        result = CHANNELS.audit_repository_environment_channels(root, files.keys())
        return result["findings"]


def codes(findings: list[dict[str, str]]) -> list[str]:
    return [item["code"] for item in findings]


class EnvironmentChannelAuditTests(unittest.TestCase):
    def test_composite_action_github_path_write_fails_closed(self) -> None:
        findings = findings_for(
            {
                "action.yml": """name: fixture\ndescription: fixture\nruns:\n  using: composite\n  steps:\n    - shell: bash\n      run: echo \"$GITHUB_ACTION_PATH/scripts\" >> \"$GITHUB_PATH\"\n"""
            }
        )
        self.assertIn("GITHUB_PATH", [item["channel"] for item in findings])

    def test_workflow_github_env_path_write_fails_closed(self) -> None:
        findings = findings_for(
            {
                ".github/workflows/test.yml": """name: test\non: pull_request\njobs:\n  test:\n    runs-on: ubuntu-24.04\n    steps:\n      - run: printf 'PATH=%s\\n' \"$GITHUB_ACTION_PATH/scripts:$PATH\" >> \"${GITHUB_ENV}\"\n"""
            }
        )
        self.assertIn("GITHUB_ENV", [item["channel"] for item in findings])

    def test_channel_name_then_indirect_parameter_expansion_fails_closed(self) -> None:
        findings = findings_for(
            {
                "scripts/build.sh": "#!/usr/bin/env bash\nchannel=GITHUB_PATH\nprintf '%s\\n' /tmp/bin >> \"${!channel}\"\n"
            }
        )
        self.assertIn("GITHUB_PATH", [item["channel"] for item in findings])
        self.assertIn("unsupported_indirect_parameter_expansion", codes(findings))

    def test_indirect_parameter_expansion_is_always_unsupported(self) -> None:
        findings = findings_for(
            {
                "scripts/build.sh": "#!/usr/bin/env bash\nchannel=SAFE_CHANNEL\nprintf '%s\\n' value >> \"${!channel}\"\n"
            }
        )
        self.assertIn("unsupported_indirect_parameter_expansion", codes(findings))

    def test_split_github_name_fragment_fails_closed(self) -> None:
        findings = findings_for(
            {
                "scripts/build.sh": "#!/usr/bin/env bash\ntarget=\"$GITHUB_\"PATH\nprintf '%s\\n' /tmp/bin >> \"$target\"\n"
            }
        )
        self.assertIn("unsupported_github_environment_name_fragment", codes(findings))

    def test_literal_github_prefix_fragment_fails_closed(self) -> None:
        findings = findings_for(
            {
                "scripts/build.sh": "#!/usr/bin/env bash\nprefix=GITHUB_\nchannel=\"${prefix}PATH\"\nprintf '%s\\n' /tmp/bin >> \"${!channel}\"\n"
            }
        )
        self.assertIn("unsupported_github_environment_name_fragment", codes(findings))
        self.assertIn("unsupported_indirect_parameter_expansion", codes(findings))

    def test_extensionless_shell_script_is_covered(self) -> None:
        findings = findings_for(
            {
                "scripts/build": "#!/usr/bin/env bash\nprintf '%s\\n' /tmp/bin >> \"${GITHUB_PATH}\"\n"
            }
        )
        self.assertIn("GITHUB_PATH", [item["channel"] for item in findings])

    def test_github_output_remains_allowed(self) -> None:
        findings = findings_for(
            {
                "scripts/github-action.sh": "#!/usr/bin/env bash\nprintf 'passed=true\\n' >> \"$GITHUB_OUTPUT\"\n"
            }
        )
        self.assertEqual([], findings)

    def test_human_diagnostic_without_special_variable_names_is_allowed(self) -> None:
        findings = findings_for(
            {
                "scripts/diagnostic.sh": "#!/usr/bin/env bash\nprintf '%s\\n' 'GitHub path and environment command files are forbidden'\n"
            }
        )
        self.assertEqual([], findings)

    def test_repeat_output_is_deterministic(self) -> None:
        files = {
            "action.yml": """name: fixture\ndescription: fixture\nruns:\n  using: composite\n  steps:\n    - shell: bash\n      run: echo /tmp/bin >> \"$GITHUB_PATH\"\n""",
            "scripts/build.sh": "#!/usr/bin/env bash\nchannel=GITHUB_ENV\nprintf 'PATH=/tmp/bin\\n' >> \"${!channel}\"\n",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for path, text in files.items():
                candidate = root / path
                candidate.parent.mkdir(parents=True, exist_ok=True)
                candidate.write_text(text, encoding="utf-8")
            first = CHANNELS.audit_repository_environment_channels(root, files.keys())
            second = CHANNELS.audit_repository_environment_channels(root, reversed(list(files.keys())))
        self.assertEqual(first, second)

    def test_live_repository_has_no_cross_step_environment_channel_authority(self) -> None:
        root = Path(__file__).resolve().parents[2]
        tracked = CHANNELS._tracked_files(root)
        result = CHANNELS.audit_repository_environment_channels(root, tracked)
        self.assertTrue(result["ok"], result["findings"])


if __name__ == "__main__":
    unittest.main()
