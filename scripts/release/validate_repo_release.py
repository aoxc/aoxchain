#!/usr/bin/env python3
"""Validate a repository release directory contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

REQUIRED_FILES = [
    "manifest.json",
    "checksums.sha256",
    "compatibility.toml",
    "binaries",
    "signatures",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate releases/v<version> directory contract.")
    parser.add_argument("release_dir", help="Path to versioned release directory, e.g. releases/v0.2.0-aoxcq")
    return parser.parse_args()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    args = parse_args()
    release_dir = Path(args.release_dir).resolve()
    if not release_dir.exists() or not release_dir.is_dir():
        raise FileNotFoundError(f"Release directory not found: {release_dir}")

    for rel in REQUIRED_FILES:
        path = release_dir / rel
        if not path.exists():
            raise FileNotFoundError(f"Missing required release entry: {path}")

    manifest_path = release_dir / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    artifacts = manifest.get("artifacts", [])
    if not artifacts:
        raise ValueError("manifest.json has no artifacts")

    checksum_lines = (release_dir / "checksums.sha256").read_text(encoding="utf-8").strip().splitlines()
    checksum_map: dict[str, str] = {}
    for line in checksum_lines:
        parts = line.split()
        if len(parts) < 2:
            raise ValueError(f"Invalid checksums.sha256 line: {line}")
        checksum_map[parts[-1]] = parts[0]

    for artifact in artifacts:
        rel_path = artifact["path"]
        expected = artifact["sha256"]
        artifact_path = release_dir / rel_path
        if not artifact_path.exists() or not artifact_path.is_file():
            raise FileNotFoundError(f"Artifact listed in manifest does not exist: {artifact_path}")

        actual = sha256(artifact_path)
        if actual != expected:
            raise ValueError(
                f"Artifact checksum mismatch for {rel_path}: expected manifest {expected}, computed {actual}"
            )

        checksum_file_value = checksum_map.get(rel_path)
        if checksum_file_value is None:
            raise ValueError(f"Artifact missing in checksums.sha256: {rel_path}")
        if checksum_file_value != actual:
            raise ValueError(
                f"checksums.sha256 mismatch for {rel_path}: listed {checksum_file_value}, computed {actual}"
            )

    print(f"Release directory is valid: {release_dir}")
    print(f"Artifacts verified: {len(artifacts)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"[validate_repo_release][error] {exc}", file=sys.stderr)
        raise SystemExit(2)
