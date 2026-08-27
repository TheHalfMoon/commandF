#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("audit_workflow_trust_environment_channels.py")
SPEC = importlib.util.spec_from_file_location(
    "audit_workflow_trust_environment_channels_precision_target", MODULE_PATH
)
assert SPEC is not None and SPEC.loader is not None
CHANNELS = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = CHANNELS
SPEC.loader.exec_module(CHANNELS)


def findings_for(script: str) -> list[dict[str, str]]:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        path = "scripts/build.sh"
        candidate = root / path
        candidate.parent.mkdir(parents=True, exist_ok=True)
        candidate.write_text("#!/usr/bin/env bash\n" + script, encoding="utf-8")
        result = CHANNELS.audit_repository_environment_channels(root, [path])
        return result["findings"]


class EnvironmentChannelPrecisionTests(unittest.TestCase):
    def test_quoted_semicolon_in_static_assignment_is_data(self) -> None:
        findings = findings_for(
            'local fsh_index="$case_dir/fsh index;literal.json"\n'
            'export REPORT_PATH="$fsh_index"\n'
        )
        self.assertEqual([], findings)

    def test_semicolon_between_commands_still_exposes_dynamic_writer(self) -> None:
        findings = findings_for(
            'prefix=BASH; export "${prefix}_ENV=/tmp/bootstrap.sh"; echo ok\n'
        )
        self.assertIn(
            "unsupported_dynamic_variable_write",
            [item["code"] for item in findings],
        )


if __name__ == "__main__":
    unittest.main()
