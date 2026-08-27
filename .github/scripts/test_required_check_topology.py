#!/usr/bin/env python3
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / ".github" / "required-checks.json"
WORKFLOWS = ROOT / ".github" / "workflows"
GITHUB_ACTIONS_INTEGRATION_ID = 15368


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


def workflow_paths(root: Path = WORKFLOWS) -> list[Path]:
    return sorted(
        path
        for path in root.iterdir()
        if path.is_file() and path.suffix in {".yml", ".yaml"}
    )


def literal_job_name(lines: list[str], start: int, end: int) -> str | None:
    names = [line[9:].strip() for line in lines[start + 1 : end] if line.startswith("    name:")]
    if len(names) > 1:
        raise AssertionError("workflow job has multiple top-level name fields")
    if not names:
        return None
    value = names[0]
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        value = value[1:-1]
    if not value:
        raise AssertionError("workflow job has an empty name")
    if "${{" in value:
        return None
    return value


def job_check_contexts(path: Path) -> list[tuple[str, str | None]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    try:
        jobs_index = lines.index("jobs:")
    except ValueError:
        return []
    job_ids: list[str] = []
    for line in lines[jobs_index + 1 :]:
        if line and not line.startswith(" "):
            break
        if line.startswith("  ") and not line.startswith("    ") and line.rstrip().endswith(":"):
            job_ids.append(line.strip()[:-1])
    result: list[tuple[str, str | None]] = []
    for job_id in job_ids:
        start, end = job_range(lines, job_id)
        explicit_name = literal_job_name(lines, start, end)
        result.append((job_id, explicit_name or job_id))
    return result


def required_context_producers(
    workflow_root: Path, contexts: set[str]
) -> dict[str, list[tuple[str, str]]]:
    producers = {context: [] for context in contexts}
    for path in workflow_paths(workflow_root):
        relative = display_path(path)
        for job_id, context in job_check_contexts(path):
            if context in contexts:
                producers[str(context)].append((relative, job_id))
    return producers


class RequiredCheckTopologyTests(unittest.TestCase):
    def load_config(self) -> dict[str, object]:
        value = json.loads(CONFIG.read_text(encoding="utf-8"))
        self.assertIsInstance(value, dict)
        return value

    def test_selected_checks_are_unique_universal_and_integration_bound(self) -> None:
        config = self.load_config()
        self.assertEqual(config.get("schema"), 1)
        self.assertEqual(config.get("protected_branch"), "main")
        checks = config.get("checks")
        self.assertIsInstance(checks, list)
        self.assertGreater(len(checks), 0)
        contexts: set[str] = set()
        pairs: set[tuple[str, str]] = set()
        expected_producers: dict[str, tuple[str, str]] = {}
        for index, item in enumerate(checks):
            with self.subTest(index=index):
                self.assertIsInstance(item, dict)
                self.assertEqual(set(item), {"context", "integration_id", "workflow", "job"})
                context = item["context"]
                integration_id = item["integration_id"]
                workflow = item["workflow"]
                job = item["job"]
                self.assertIsInstance(context, str)
                self.assertEqual(integration_id, GITHUB_ACTIONS_INTEGRATION_ID)
                self.assertIsInstance(workflow, str)
                self.assertIsInstance(job, str)
                self.assertNotIn(context, contexts)
                self.assertNotIn((workflow, job), pairs)
                contexts.add(context)
                pairs.add((workflow, job))
                expected_producers[context] = (workflow, job)
                path = ROOT / workflow
                self.assertTrue(path.is_file(), workflow)
                assert_universal_required_job(path, job)
                actual_contexts = dict(job_check_contexts(path))
                self.assertEqual(
                    actual_contexts.get(job),
                    context,
                    f"{workflow} job {job!r} does not emit required context {context!r}",
                )

        producers = required_context_producers(WORKFLOWS, contexts)
        for context, expected in expected_producers.items():
            self.assertEqual(
                producers[context],
                [expected],
                f"required context {context!r} must have exactly one authoritative workflow/job producer",
            )

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

    def test_counterexample_duplicate_or_named_spoof_context_is_detected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "authoritative.yml").write_text(
                "name: authoritative\njobs:\n  rust:\n    runs-on: ubuntu-24.04\n",
                encoding="utf-8",
            )
            (root / "spoof.yml").write_text(
                "name: spoof\njobs:\n  harmless-id:\n    name: rust\n    runs-on: ubuntu-24.04\n",
                encoding="utf-8",
            )
            producers = required_context_producers(root, {"rust"})
            self.assertEqual(len(producers["rust"]), 2)
            self.assertEqual(
                producers["rust"],
                [
                    (str(root / "authoritative.yml"), "rust"),
                    (str(root / "spoof.yml"), "harmless-id"),
                ],
            )


if __name__ == "__main__":
    unittest.main()
