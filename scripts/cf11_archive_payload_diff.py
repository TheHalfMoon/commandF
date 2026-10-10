#!/usr/bin/env python3
"""Offline, bounded TAR payload comparison; diagnostics, never a release oracle."""
from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import sys
import tarfile
from pathlib import Path

MAX_ARCHIVE_BYTES = 128 * 1024 * 1024
MAX_DECOMPRESSED_BYTES = 896 * 1024 * 1024
MAX_MEMBERS = 50_000
MAX_MANIFEST_BYTES = 1024 * 1024
BLOCK_BYTES = 1024 * 1024
KNOWN_MANIFEST_FIELDS = frozenset({
    "name", "version", "dependencies", "url", "notForPublication",
    "fhirVersions", "fhir-version-list",
})


class InvalidArchive(Exception):
    """An input violates the intentionally strict archive-evidence contract."""


class LimitedDecodedStream:
    """Count real inflated bytes, including TAR headers and skipped payloads."""

    def __init__(self, source, limit=None):
        self.source = source
        self.limit = MAX_DECOMPRESSED_BYTES if limit is None else limit
        self.total = 0

    def read(self, size=-1):
        if size < 0:
            raise InvalidArchive("unbounded decompression read")
        block = self.source.read(min(size, self.limit - self.total + 1))
        self.total += len(block)
        if self.total > self.limit:
            raise InvalidArchive("decompressed byte limit")
        return block


def canonical_member_name(raw: str) -> str:
    while raw.startswith("./"):
        raw = raw[2:]
    parts = raw.split("/")
    if (not raw or raw.startswith("/") or "\\" in raw or "\x00" in raw
            or any(part in ("", ".", "..") for part in parts)):
        raise InvalidArchive("unsafe TAR entry path")
    return raw


def manifest_unique_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise InvalidArchive("duplicate package manifest JSON key")
        result[key] = value
    return result


def member_metadata_digest(member: tarfile.TarInfo) -> str:
    """Hash security-relevant TAR metadata without exposing its raw values."""
    fields = {
        "type": member.type.decode("ascii", "backslashreplace"),
        "mode": member.mode,
        "uid": member.uid,
        "gid": member.gid,
        "uname": member.uname,
        "gname": member.gname,
        "mtime": member.mtime,
        "linkname": member.linkname,
        "devmajor": member.devmajor,
        "devminor": member.devminor,
        "pax_headers": member.pax_headers,
    }
    canonical = json.dumps(fields, sort_keys=True, ensure_ascii=True)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def archive_index(path: Path) -> dict:
    size = path.stat().st_size
    if size == 0 or size > MAX_ARCHIVE_BYTES:
        raise InvalidArchive("compressed archive size limit")
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(BLOCK_BYTES), b""):
            digest.update(block)
    files = {}
    file_metadata = {}
    directories = {}
    manifest = None
    member_count = 0
    with path.open("rb") as raw:
        with gzip.GzipFile(fileobj=raw) as gz:
            decoded = LimitedDecodedStream(gz)
            with tarfile.open(fileobj=decoded, mode="r|") as tar:
                for member in tar:
                    member_count += 1
                    if member_count > MAX_MEMBERS:
                        raise InvalidArchive("TAR member count limit")
                    name = canonical_member_name(member.name.rstrip("/") if member.isdir()
                                                 else member.name)
                    if name in files or name in directories:
                        raise InvalidArchive("duplicate normalized TAR member")
                    metadata_digest = member_metadata_digest(member)
                    if member.isdir():
                        directories[name] = metadata_digest
                        continue
                    if not member.isfile():
                        raise InvalidArchive("unsupported TAR member type")
                    if name == "package/package.json" and member.size > MAX_MANIFEST_BYTES:
                        raise InvalidArchive("manifest byte limit")
                    stream = tar.extractfile(member)
                    if stream is None:
                        raise InvalidArchive("unreadable TAR member")
                    content_digest = hashlib.sha256()
                    manifest_bytes = bytearray() if name == "package/package.json" else None
                    while True:
                        block = stream.read(BLOCK_BYTES)
                        if not block:
                            break
                        content_digest.update(block)
                        if manifest_bytes is not None:
                            manifest_bytes.extend(block)
                            if len(manifest_bytes) > MAX_MANIFEST_BYTES:
                                raise InvalidArchive("manifest byte limit")
                    files[name] = content_digest.hexdigest()
                    file_metadata[name] = metadata_digest
                    if manifest_bytes is not None:
                        try:
                            manifest = json.loads(
                                manifest_bytes, object_pairs_hook=manifest_unique_keys
                            )
                        except (ValueError, UnicodeDecodeError) as exc:
                            raise InvalidArchive("invalid package manifest JSON") from exc
                        if (not isinstance(manifest, dict)
                                or any(not isinstance(manifest.get(key), str)
                                       or not manifest[key] for key in ("name", "version"))):
                            raise InvalidArchive("invalid package manifest identity")
            # Force gzip trailer/CRC validation and count bytes beyond the TAR terminator.
            while decoded.read(BLOCK_BYTES):
                pass
    if manifest is None:
        raise InvalidArchive("missing package manifest")
    return {"archive_sha256": digest.hexdigest(), "files": files,
            "file_metadata": file_metadata, "directories": directories,
            "members": member_count, "manifest": manifest}


def compare_archives(first: Path, second: Path) -> dict:
    a, b = archive_index(first), archive_index(second)
    a_names, b_names = a["files"].keys(), b["files"].keys()
    common = a_names & b_names
    content_equal = a["files"] == b["files"]
    metadata_equal = (
        a["file_metadata"] == b["file_metadata"]
        and a["directories"] == b["directories"]
    )
    dir_names_a, dir_names_b = a["directories"].keys(), b["directories"].keys()
    common_dirs = dir_names_a & dir_names_b
    bytes_equal = a["archive_sha256"] == b["archive_sha256"]
    af, bf = a["manifest"], b["manifest"]
    absent = object()
    differing = {key for key in af.keys() | bf.keys()
                 if af.get(key, absent) != bf.get(key, absent)}
    # Do not expose file names, manifest field values, URLs or local paths.
    return {
        "schema": "commandf.cf11-archive-payload-comparison/v1",
        "overall": ("ARCHIVE_BYTES_IDENTICAL" if bytes_equal
                    else "ARCHIVE_BYTES_DIFFERENT_CONTENT_IDENTICAL"
                    if content_equal and metadata_equal
                    else "ARCHIVE_METADATA_DIVERGENCE" if content_equal
                    else "ARCHIVE_CONTENT_DIVERGENCE"),
        "archive_sha256_first": a["archive_sha256"],
        "archive_sha256_second": b["archive_sha256"],
        "manifest_name_version_equal": (
            af.get("name") == bf.get("name") and af.get("version") == bf.get("version")
        ),
        "manifest_dependencies_equal": af.get("dependencies") == bf.get("dependencies"),
        "manifest_selected_changed_fields": sorted(differing & KNOWN_MANIFEST_FIELDS),
        "manifest_other_fields_changed": bool(differing - KNOWN_MANIFEST_FIELDS),
        "tar_members": {"first": a["members"], "second": b["members"]},
        "regular_files": {"first": len(a_names), "second": len(b_names)},
        "directories": {"first": len(dir_names_a), "second": len(dir_names_b)},
        "first_only_directories": len(dir_names_a - dir_names_b),
        "second_only_directories": len(dir_names_b - dir_names_a),
        "common_changed_directory_metadata": sum(
            a["directories"][name] != b["directories"][name] for name in common_dirs
        ),
        "common_changed_file_metadata": sum(
            a["file_metadata"][name] != b["file_metadata"][name] for name in common
        ),
        "first_only_files": len(a_names - b_names),
        "second_only_files": len(b_names - a_names),
        "common_changed_files": sum(a["files"][name] != b["files"][name] for name in common),
    }


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--first", required=True, type=Path, help="first local archive")
    parser.add_argument("--second", required=True, type=Path, help="second local archive")
    args = parser.parse_args(argv)
    try:
        report = compare_archives(args.first, args.second)
    except (InvalidArchive, OSError, EOFError, gzip.BadGzipFile,
            tarfile.TarError, OverflowError, ValueError, RecursionError):
        # Input errors are deliberately generic to avoid leaking host paths.
        print(json.dumps({"schema": "commandf.cf11-archive-payload-comparison/v1",
                          "overall": "INVALID_ARCHIVE"}))
        return 4
    print(json.dumps(report, sort_keys=True, indent=2))
    return 0 if report["overall"] == "ARCHIVE_BYTES_IDENTICAL" else 2


if __name__ == "__main__":
    raise SystemExit(main())
