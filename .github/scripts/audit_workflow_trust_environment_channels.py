#!/usr/bin/env python3
"""Fail closed on GitHub environment channels that can change later-step command resolution."""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Iterable

CHANNEL_RE = re.compile(
    r"\$(?P<plain>GITHUB_PATH|GITHUB_ENV)(?![A-Za-z0-9_])"
    r"|\$\{(?P<braced>GITHUB_PATH|GITHUB_ENV)(?=[^A-Za-z0-9_}]|\})"
)
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


def audit_repository_environment_channels(
    root: Path, tracked_files: Iterable[str]
) -> dict[str, object]:
    """Reject GitHub PATH/ENV command channels in tracked workflow and shell authority."""
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
        for matched in CHANNEL_RE.finditer(text):
            channel = matched.group("plain") or matched.group("braced") or ""
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
