#!/usr/bin/env python3
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".github" / "required-checks.json"


def display_path(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def pull_request_children(lines: list[str]) -> list[str]:
    try:
        start = lines.index("  pull_request:")
    except ValueError as error:
        raise AssertionError("required-check workflow must use a mapping-form pull_request trigger") from error
    children: list[str] = []
    for line in lines[start + 1 :]:
        if line and not line.startswith(" "):
            break
        if line.startswith("  ") and not line.startswith("    ") and line.strip():
            break
        if line.strip() and not line.lstrip().startswith("#"):
            children.append(line.strip())
    return children


def job_range(lines: list[str], job: str) -> tuple[int, int]:
    try:
        jobs_index = lines.index("jobs:")
    except ValueError as error:
        raise AssertionError("required-check workflow has no jobs mapping") from error
    target = f"  {job}:"
    try:
        start = lines.index(target, jobs_index + 1)
    except ValueError as error:
        raise AssertionError(f"required-check workflow has no job {job!r}") from error
    end = len(lines)
    for index in range(start + 1, len(lines)):
        line = lines[index]
        if line and not line.startswith(" "):
            end = index
            break
        if line.startswith("  ") and not line.startswith("    ") and line.strip():
            end = index
            break
    return start, end


def assert_universal_required_job(path: Path, job: str) -> None:
    lines = path.read_text(encoding="utf-8").splitlines()
    children = pull_request_children(lines)
    if children:
        raise AssertionError(
            f"{display_path(path)} pull_request trigger is filtered or narrowed: {children!r}"
        )
    start, end = job_range(lines, job)
    for line in lines[start + 1 : end]:
        if line.startswith("    if:"):
            raise AssertionError(
                f"{display_path(path)} job {job!r} has a job-level conditional and is not universally terminal"
            )


class RequiredCheckTopologyTests(unittest.TestCase):
    def load_config(self) -> dict[str, object]:
        value = json.loads(CONFIG.read_text(encoding="utf-8"))
        self.assertIsInstance(value, dict)
        return value

    def test_selected_checks_are_unique_and_universal(self) -> None:
        config = self.load_config()
        self.assertEqual(config.get("schema"), 1)
        self.assertEqual(config.get("protected_branch"), "main")
        checks = config.get("checks")
        self.assertIsInstance(checks, list)
        self.assertGreater(len(checks), 0)
        contexts: set[str] = set()
        pairs: set[tuple[str, str]] = set()
        for index, item in enumerate(checks):
            with self.subTest(index=index):
                self.assertIsInstance(item, dict)
                self.assertEqual(set(item), {"context", "workflow", "job"})
                context = item["context"]
                workflow = item["workflow"]
                job = item["job"]
                self.assertIsInstance(context, str)
                self.assertIsInstance(workflow, str)
                self.assertIsInstance(job, str)
                self.assertNotIn(context, contexts)
                self.assertNotIn((workflow, job), pairs)
                contexts.add(context)
                pairs.add((workflow, job))
                path = ROOT / workflow
                self.assertTrue(path.is_file(), workflow)
                assert_universal_required_job(path, job)

    def test_counterexample_path_filtered_workflow_is_rejected(self) -> None:
        lines = [
            "name: example",
            "on:",
            "  pull_request:",
            "    paths:",
            "      - src/**",
            "jobs:",
            "  gate:",
            "    runs-on: ubuntu-24.04",
        ]
        self.assertEqual(pull_request_children(lines), ["paths:", "- src/**"])

    def test_counterexample_job_level_if_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "workflow.yml"
            path.write_text(
                "name: example\non:\n  pull_request:\njobs:\n  gate:\n    if: github.actor != 'x'\n    runs-on: ubuntu-24.04\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(AssertionError, "job-level conditional"):
                assert_universal_required_job(path, "gate")


if __name__ == "__main__":
    unittest.main()
