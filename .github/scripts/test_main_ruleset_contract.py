#!/usr/bin/env python3
from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TOPOLOGY = ROOT / ".github" / "required-checks.json"
ASSURANCE_RULESET = ROOT / ".github" / "main-ruleset.json"
REVIEW_RULESET = ROOT / ".github" / "main-review-ruleset.json"
GITHUB_ACTIONS_INTEGRATION_ID = 15368
ADMIN_REPOSITORY_ROLE_ID = 5
MAIN_CONDITIONS = {"ref_name": {"include": ["refs/heads/main"], "exclude": []}}


class MainRulesetContractTests(unittest.TestCase):
    def test_assurance_ruleset_is_unbypassable_and_matches_required_checks(self) -> None:
        topology = json.loads(TOPOLOGY.read_text(encoding="utf-8"))
        ruleset = json.loads(ASSURANCE_RULESET.read_text(encoding="utf-8"))

        self.assertEqual(ruleset.get("name"), "commandF main assurance")
        self.assertEqual(ruleset.get("target"), "branch")
        self.assertEqual(ruleset.get("enforcement"), "active")
        self.assertEqual(ruleset.get("bypass_actors"), [])
        self.assertEqual(ruleset.get("conditions"), MAIN_CONDITIONS)

        rules = ruleset.get("rules")
        self.assertIsInstance(rules, list)
        by_type = {rule.get("type"): rule for rule in rules if isinstance(rule, dict)}
        self.assertEqual(set(by_type), {"deletion", "non_fast_forward", "required_status_checks"})
        self.assertEqual(by_type["deletion"], {"type": "deletion"})
        self.assertEqual(by_type["non_fast_forward"], {"type": "non_fast_forward"})

        required = by_type["required_status_checks"].get("parameters")
        self.assertIsInstance(required, dict)
        self.assertFalse(required.get("do_not_enforce_on_create"))
        self.assertTrue(required.get("strict_required_status_checks_policy"))
        actual = required.get("required_status_checks", [])
        expected = [
            {"context": item["context"], "integration_id": item["integration_id"]}
            for item in topology["checks"]
        ]
        self.assertEqual(actual, expected)
        self.assertEqual(
            actual,
            [
                {"context": "rust", "integration_id": GITHUB_ACTIONS_INTEGRATION_ID},
                {"context": "assurance-proof", "integration_id": GITHUB_ACTIONS_INTEGRATION_ID},
                {"context": "scorecard", "integration_id": GITHUB_ACTIONS_INTEGRATION_ID},
            ],
        )

    def test_review_ruleset_requires_review_with_admin_pr_only_escape_hatch(self) -> None:
        ruleset = json.loads(REVIEW_RULESET.read_text(encoding="utf-8"))
        self.assertEqual(ruleset.get("name"), "commandF main review governance")
        self.assertEqual(ruleset.get("target"), "branch")
        self.assertEqual(ruleset.get("enforcement"), "active")
        self.assertEqual(ruleset.get("conditions"), MAIN_CONDITIONS)
        self.assertEqual(
            ruleset.get("bypass_actors"),
            [
                {
                    "actor_id": ADMIN_REPOSITORY_ROLE_ID,
                    "actor_type": "RepositoryRole",
                    "bypass_mode": "pull_request",
                }
            ],
        )

        rules = ruleset.get("rules")
        self.assertEqual(len(rules), 1)
        self.assertEqual(rules[0].get("type"), "pull_request")
        self.assertEqual(
            rules[0].get("parameters"),
            {
                "allowed_merge_methods": ["merge"],
                "dismiss_stale_reviews_on_push": True,
                "require_code_owner_review": True,
                "require_last_push_approval": True,
                "required_approving_review_count": 1,
                "required_review_thread_resolution": True,
            },
        )

    def test_admin_review_bypass_cannot_bypass_assurance_rules(self) -> None:
        assurance = json.loads(ASSURANCE_RULESET.read_text(encoding="utf-8"))
        review = json.loads(REVIEW_RULESET.read_text(encoding="utf-8"))
        self.assertEqual(assurance["bypass_actors"], [])
        self.assertNotIn("pull_request", {rule["type"] for rule in assurance["rules"]})
        self.assertEqual({rule["type"] for rule in review["rules"]}, {"pull_request"})
        self.assertNotIn("required_status_checks", {rule["type"] for rule in review["rules"]})
        self.assertNotIn("deletion", {rule["type"] for rule in review["rules"]})
        self.assertNotIn("non_fast_forward", {rule["type"] for rule in review["rules"]})

    def test_every_required_check_is_bound_to_github_actions(self) -> None:
        topology = json.loads(TOPOLOGY.read_text(encoding="utf-8"))
        for check in topology["checks"]:
            self.assertEqual(check.get("integration_id"), GITHUB_ACTIONS_INTEGRATION_ID)

        ruleset = json.loads(ASSURANCE_RULESET.read_text(encoding="utf-8"))
        required_rule = next(
            rule for rule in ruleset["rules"] if rule["type"] == "required_status_checks"
        )
        for check in required_rule["parameters"]["required_status_checks"]:
            self.assertEqual(set(check), {"context", "integration_id"})
            self.assertEqual(check["integration_id"], GITHUB_ACTIONS_INTEGRATION_ID)


if __name__ == "__main__":
    unittest.main()
