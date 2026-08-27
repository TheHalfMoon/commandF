#!/usr/bin/env python3
"""Fail closed on GitHub environment channels that can change later-step command resolution."""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Iterable

CHANNEL_NAME_RE = re.compile(
    r"(?<![A-Za-z0-9_])(?P<channel>GITHUB_PATH|GITHUB_ENV)(?![A-Za-z0-9_])"
)
INDIRECT_PARAMETER_RE = re.compile(r"\$\{!")
GITHUB_PREFIX_FRAGMENT_RE = re.compile(r"(?<![A-Za-z0-9_])GITHUB_(?![A-Za-z0-9_])")
SHELL_SHEBANG_RE = re.compile(r"^#![^\n]*\b(?:bash|dash|ksh|sh|zsh)\b")


def _tracked_files(root: Path) -> list[str]:
    completed = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z"],
        check=False,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise RuntimeError("unable to enumerate tracked repository files")
    return sorted(
        item.decode("utf-8")
        for item in completed.stdout.split(b"\0")
        if item
    )


def _is_yaml_authority(path: str) -> bool:
    candidate = Path(path)
    return (
        path.startswith(".github/workflows/") and candidate.suffix in {".yml", ".yaml"}
    ) or candidate.name in {"action.yml", "action.yaml"}


def _read_authority_text(root: Path, path: str) -> tuple[str | None, str | None]:
    candidate = root / path
    try:
        raw = candidate.read_bytes()
    except OSError as error:
        if _is_yaml_authority(path) or candidate.suffix == ".sh":
            return None, f"unable to read tracked authority file: {error}"
        return None, None

    is_shell = candidate.suffix == ".sh"
    if not is_shell:
        prefix = raw[:256].decode("utf-8", errors="ignore")
        is_shell = SHELL_SHEBANG_RE.match(prefix) is not None
    if not (_is_yaml_authority(path) or is_shell):
        return None, None

    try:
        return raw.decode("utf-8"), None
    except UnicodeDecodeError:
        return None, "tracked authority file is not valid UTF-8"


def _channel_findings(path: str, text: str) -> list[dict[str, str]]:
    findings: list[dict[str, str]] = []
    for matched in CHANNEL_NAME_RE.finditer(text):
        channel = matched.group("channel")
        findings.append(
            {
                "channel": channel,
                "code": "unsupported_github_environment_channel",
                "detail": (
                    f"{channel} can mutate later-step environment/command resolution and "
                    "is outside the AF-01 constrained shell authority"
                ),
                "path": path,
            }
        )
    if INDIRECT_PARAMETER_RE.search(text) is not None:
        findings.append(
            {
                "channel": "",
                "code": "unsupported_indirect_parameter_expansion",
                "detail": (
                    "indirect shell parameter expansion can resolve a forbidden GitHub "
                    "environment channel and is outside AF-01 authority"
                ),
                "path": path,
            }
        )
    if GITHUB_PREFIX_FRAGMENT_RE.search(text) is not None:
        findings.append(
            {
                "channel": "",
                "code": "unsupported_github_environment_name_fragment",
                "detail": (
                    "standalone GITHUB_ name fragments can construct a forbidden GitHub "
                    "environment channel and are outside AF-01 authority"
                ),
                "path": path,
            }
        )
    return findings


def audit_repository_environment_channels(
    root: Path, tracked_files: Iterable[str]
) -> dict[str, object]:
    """Reject direct or indirect GitHub PATH/ENV channels in tracked shell authority."""
    findings: list[dict[str, str]] = []
    for path in sorted(set(tracked_files)):
        text, read_error = _read_authority_text(root, path)
        if read_error is not None:
            findings.append(
                {
                    "channel": "",
                    "code": "unreadable_environment_channel_authority",
                    "detail": read_error,
                    "path": path,
                }
            )
            continue
        if text is None:
            continue
        findings.extend(_channel_findings(path, text))
    findings.sort(key=lambda item: (item["path"], item["channel"], item["code"], item["detail"]))
    return {"findings": findings, "ok": not findings, "schema": 1}


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    try:
        tracked = _tracked_files(root)
        result = audit_repository_environment_channels(root, tracked)
    except RuntimeError as error:
        result = {
            "findings": [
                {
                    "channel": "",
                    "code": "environment_channel_inventory_failed",
                    "detail": str(error),
                    "path": "",
                }
            ],
            "ok": False,
            "schema": 1,
        }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
