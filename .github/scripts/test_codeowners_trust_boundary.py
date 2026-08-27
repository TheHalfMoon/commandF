#!/usr/bin/env python3
from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CODEOWNERS = ROOT / ".github" / "CODEOWNERS"
RULESET = ROOT / ".github" / "main-ruleset.json"
EXPECTED_OWNER = "@TheHalfMoon"
EXPECTED_PATTERN = "/.github/"


def ownership_entries() -> list[tuple[str, tuple[str, ...]]]:
    entries: list[tuple[str, tuple[str, ...]]] = []
    for raw in CODEOWNERS.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        fields = line.split()
        if len(fields) < 2:
            raise AssertionError(f"malformed CODEOWNERS entry: {line!r}")
        entries.append((fields[0], tuple(fields[1:])))
    return entries


class CodeownersTrustBoundaryTests(unittest.TestCase):
    def test_github_security_surface_has_one_unambiguous_owner_boundary(self) -> None:
        self.assertTrue(CODEOWNERS.is_file())
        self.assertEqual(
            ownership_entries(),
            [(EXPECTED_PATTERN, (EXPECTED_OWNER,))],
            "the .github trust boundary must not contain narrower override patterns",
        )

    def test_ruleset_requires_code_owner_review_for_owned_workflow_changes(self) -> None:
        ruleset = json.loads(RULESET.read_text(encoding="utf-8"))
        pull_request = next(rule for rule in ruleset["rules"] if rule["type"] == "pull_request")
        parameters = pull_request["parameters"]
        self.assertIs(parameters.get("require_code_owner_review"), True)
        self.assertIs(parameters.get("dismiss_stale_reviews_on_push"), True)
        self.assertIs(parameters.get("require_last_push_approval"), True)
        self.assertGreaterEqual(parameters.get("required_approving_review_count", 0), 1)

    def test_codeowners_file_is_inside_the_owned_boundary(self) -> None:
        self.assertEqual(CODEOWNERS.relative_to(ROOT).as_posix(), ".github/CODEOWNERS")
        pattern, owners = ownership_entries()[0]
        self.assertEqual(pattern, EXPECTED_PATTERN)
        self.assertEqual(owners, (EXPECTED_OWNER,))


if __name__ == "__main__":
    unittest.main()
