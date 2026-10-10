#!/usr/bin/env python3
"""Offline, bounded content-difference report for two pre-fetched FHIR tgz archives.

Never downloads URLs, extracts archive paths, rewrites package content, or
equates different raw SHA-256 identities.
"""
from __future__ import annotations

import argparse
import io
import hashlib
import json
from pathlib import Path
import sys
import tarfile

MAX_ARCHIVE_BYTES = 128 * 1024 * 1024
MAX_MEMBERS = 50_000
MAX_EXPANDED_BYTES = 512 * 1024 * 1024
MAX_MEMBER_BYTES = 64 * 1024 * 1024
MAX_DISPLAY = 25
MAX_MEMBER_PATH_CHARS = 4096
CHUNK = 1024 * 1024


class AuditError(Exception):
    pass


def inspect(path: Path) -> tuple[dict, dict[str, tuple[str, tuple]]]:
    try:
        with path.open("rb") as stream:
            # One bounded immutable snapshot: digest and TAR parser see same bytes.
            compressed = stream.read(MAX_ARCHIVE_BYTES + 1)
    except OSError as error:
        raise AuditError("archive is unavailable or unreadable") from error
    if len(compressed) > MAX_ARCHIVE_BYTES:
        raise AuditError("archive must stay within the compressed byte bound")
    digest = hashlib.sha256(compressed)
    members: dict[str, tuple[str, tuple]] = {}
    counts = {"members": 0, "regular_files": 0, "nonfiles": 0}
    total_expanded = 0
    manifest_sha = None
    manifest = None

    try:
        with tarfile.open(fileobj=io.BytesIO(compressed), mode="r:gz") as archive:
            for item in archive:
                counts["members"] += 1
                if counts["members"] > MAX_MEMBERS:
                    raise AuditError("archive member-count bound exceeded")
                name = item.name
                if (
                    not name
                    or len(name) > MAX_MEMBER_PATH_CHARS
                    or name.startswith("/")
                    or "\\" in name
                    or any(segment in ("", ".", "..") for segment in name.split("/"))
                    or name in members
                ):
                    raise AuditError("unsafe or duplicate archive member name")
                meta = (
                    item.type.hex(), item.size, item.mode, item.mtime,
                    item.uid, item.gid, item.uname, item.gname
                )
                if item.isfile():
                    counts["regular_files"] += 1
                    if item.size > MAX_MEMBER_BYTES:
                        raise AuditError("individual expanded member bound exceeded")
                    total_expanded += item.size
                    if total_expanded > MAX_EXPANDED_BYTES:
                        raise AuditError("aggregate expanded member bound exceeded")
                    stream = archive.extractfile(item)
                    if stream is None:
                        raise AuditError("regular archive member is unreadable")
                    content_hash = hashlib.sha256()
                    raw_manifest = bytearray() if name == "package/package.json" else None
                    remaining = item.size
                    while remaining:
                        chunk = stream.read(min(CHUNK, remaining))
                        if not chunk:
                            raise AuditError("truncated member content")
                        remaining -= len(chunk)
                        content_hash.update(chunk)
                        if raw_manifest is not None:
                            raw_manifest.extend(chunk)
                    if raw_manifest is not None:
                        manifest_sha = content_hash.hexdigest()
                        manifest = json.loads(raw_manifest)
                    members[name] = (content_hash.hexdigest(), meta)
                else:
                    counts["nonfiles"] += 1
                    members[name] = ("", meta)
    except (tarfile.TarError, OSError, ValueError, json.JSONDecodeError) as error:
        raise AuditError("archive or package manifest could not be parsed") from error

    if not isinstance(manifest, dict):
        raise AuditError("archive has no valid package/package.json manifest")
    if not isinstance(manifest.get("name"), str) or not isinstance(manifest.get("version"), str):
        raise AuditError("package manifest identity is invalid")
    result = {
        "compressed_bytes": len(compressed),
        "raw_sha256": digest.hexdigest(),
        **counts,
        "package_name": manifest["name"],
        "package_version": manifest["version"],
        "manifest_sha256": manifest_sha,
        # Emit a stable digest rather than unbounded or sensitive manifest values.
        "declared_dependencies_sha256": hashlib.sha256(
            json.dumps(manifest.get("dependencies", {}),
                       sort_keys=True, separators=(",", ":")).encode("utf-8")
        ).hexdigest(),
    }
    return result, members


def compare(first: Path, second: Path) -> dict:
    a, ma = inspect(first)
    b, mb = inspect(second)
    if a["package_name"] != b["package_name"] or a["package_version"] != b["package_version"]:
        raise AuditError("both archives must have the same declared package identity")
    sa, sb = set(ma), set(mb)
    changed = sorted(path for path in sa & sb if ma[path][0] != mb[path][0])
    metadata = sorted(path for path in sa & sb if ma[path][1] != mb[path][1])
    first_only = sorted(sa - sb)
    second_only = sorted(sb - sa)
    return {
        "schema": "commandf.cf11-official-origin-archive-audit.v1",
        "first": a,
        "second": b,
        "raw_archives_identical": a["raw_sha256"] == b["raw_sha256"],
        "declared_dependencies_equal":
            a["declared_dependencies_sha256"] == b["declared_dependencies_sha256"],
        "first_only_paths_count": len(first_only),
        "second_only_paths_count": len(second_only),
        "first_only_regular_files_count": sum(bool(ma[path][0]) for path in first_only),
        "second_only_regular_files_count": sum(bool(mb[path][0]) for path in second_only),
        "common_content_difference_count": len(changed),
        "common_metadata_difference_count": len(metadata),
        "examples_truncated_at": MAX_DISPLAY,
        "first_only_examples": first_only[:MAX_DISPLAY],
        "second_only_examples": second_only[:MAX_DISPLAY],
        "common_content_difference_examples": changed[:MAX_DISPLAY],
        "common_metadata_difference_examples": metadata[:MAX_DISPLAY],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("first", type=Path, help="first previously fetched .tgz file")
    parser.add_argument("second", type=Path, help="second previously fetched .tgz file")
    args = parser.parse_args()
    try:
        output = compare(args.first, args.second)
    except AuditError as error:
        print(f"CF11_ARCHIVE_AUDIT_REJECTED: {error}", file=sys.stderr)
        return 1
    print(json.dumps(output, sort_keys=True, indent=2))
    return 0 if output["raw_archives_identical"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
