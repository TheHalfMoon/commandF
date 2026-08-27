#!/usr/bin/env python3
from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CODEOWNERS = ROOT / ".github" / "CODEOWNERS"
ASSURANCE_RULESET = ROOT / ".github" / "main-ruleset.json"
REVIEW_RULESET = ROOT / ".github" / "main-review-ruleset.json"
EXPECTED_OWNER = "@TheHalfMoon"
EXPECTED_PATTERN = "/.github/"
ADMIN_REPOSITORY_ROLE_ID = 5


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

    def test_review_ruleset_requires_code_owner_review(self) -> None:
        ruleset = json.loads(REVIEW_RULESET.read_text(encoding="utf-8"))
        pull_request = next(rule for rule in ruleset["rules"] if rule["type"] == "pull_request")
        parameters = pull_request["parameters"]
        self.assertIs(parameters.get("require_code_owner_review"), True)
        self.assertIs(parameters.get("dismiss_stale_reviews_on_push"), True)
        self.assertIs(parameters.get("require_last_push_approval"), True)
        self.assertGreaterEqual(parameters.get("required_approving_review_count", 0), 1)

    def test_admin_escape_hatch_is_pr_only_and_cannot_bypass_assurance(self) -> None:
        assurance = json.loads(ASSURANCE_RULESET.read_text(encoding="utf-8"))
        review = json.loads(REVIEW_RULESET.read_text(encoding="utf-8"))
        self.assertEqual(assurance.get("bypass_actors"), [])
        self.assertEqual(
            review.get("bypass_actors"),
            [
                {
                    "actor_id": ADMIN_REPOSITORY_ROLE_ID,
                    "actor_type": "RepositoryRole",
                    "bypass_mode": "pull_request",
                }
            ],
        )

    def test_codeowners_file_is_inside_the_owned_boundary(self) -> None:
        self.assertEqual(CODEOWNERS.relative_to(ROOT).as_posix(), ".github/CODEOWNERS")
        pattern, owners = ownership_entries()[0]
        self.assertEqual(pattern, EXPECTED_PATTERN)
        self.assertEqual(owners, (EXPECTED_OWNER,))


if __name__ == "__main__":
    unittest.main()
