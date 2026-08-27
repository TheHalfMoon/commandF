#!/usr/bin/env python3
from __future__ import annotations

import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TOPOLOGY = ROOT / ".github" / "required-checks.json"
RULESET = ROOT / ".github" / "main-ruleset.json"


class MainRulesetContractTests(unittest.TestCase):
    def test_ruleset_matches_required_check_topology_and_governance(self) -> None:
        topology = json.loads(TOPOLOGY.read_text(encoding="utf-8"))
        ruleset = json.loads(RULESET.read_text(encoding="utf-8"))

        self.assertEqual(ruleset.get("name"), "commandF main assurance")
        self.assertEqual(ruleset.get("target"), "branch")
        self.assertEqual(ruleset.get("enforcement"), "active")
        self.assertEqual(ruleset.get("bypass_actors"), [])
        self.assertEqual(
            ruleset.get("conditions"),
            {"ref_name": {"include": ["refs/heads/main"], "exclude": []}},
        )

        rules = ruleset.get("rules")
        self.assertIsInstance(rules, list)
        by_type = {rule.get("type"): rule for rule in rules if isinstance(rule, dict)}
        self.assertEqual(set(by_type), {"deletion", "non_fast_forward", "pull_request", "required_status_checks"})
        self.assertEqual(by_type["deletion"], {"type": "deletion"})
        self.assertEqual(by_type["non_fast_forward"], {"type": "non_fast_forward"})

        pull_request = by_type["pull_request"].get("parameters")
        self.assertEqual(
            pull_request,
            {
                "allowed_merge_methods": ["merge"],
                "dismiss_stale_reviews_on_push": True,
                "require_code_owner_review": False,
                "require_last_push_approval": True,
                "required_approving_review_count": 1,
                "required_review_thread_resolution": True,
            },
        )

        required = by_type["required_status_checks"].get("parameters")
        self.assertIsInstance(required, dict)
        self.assertFalse(required.get("do_not_enforce_on_create"))
        self.assertTrue(required.get("strict_required_status_checks_policy"))
        contexts = [item.get("context") for item in required.get("required_status_checks", [])]
        expected = [item["context"] for item in topology["checks"]]
        self.assertEqual(contexts, expected)
        self.assertEqual(contexts, ["rust", "assurance-proof", "scorecard"])

    def test_ruleset_has_no_unreviewed_integration_binding(self) -> None:
        ruleset = json.loads(RULESET.read_text(encoding="utf-8"))
        required_rule = next(rule for rule in ruleset["rules"] if rule["type"] == "required_status_checks")
        for check in required_rule["parameters"]["required_status_checks"]:
            self.assertEqual(set(check), {"context"})


if __name__ == "__main__":
    unittest.main()
