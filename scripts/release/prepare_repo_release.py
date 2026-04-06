#!/usr/bin/env python3
"""Prepare a repository-local versioned release directory.

Creates `releases/v<version>/` and core publication metadata for AOXC binaries.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


@dataclass(frozen=True)
class NetworkIdentity:
    network_key: str
    network_id: str
    chain_id: str


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def extract_toml_section_key(text: str, section: str, key: str) -> str:
    section_pattern = re.compile(rf"^\[{re.escape(section)}\]\s*$", re.MULTILINE)
    match = section_pattern.search(text)
    if not match:
        raise ValueError(f"Section not found: [{section}]")

    start = match.end()
    next_section = re.search(r"^\[[^\]]+\]\s*$", text[start:], re.MULTILINE)
    end = start + next_section.start() if next_section else len(text)
    body = text[start:end]

    key_pattern = re.compile(rf"^\s*{re.escape(key)}\s*=\s*(.+?)\s*$", re.MULTILINE)
    key_match = key_pattern.search(body)
    if not key_match:
        raise ValueError(f"Key not found: {key} in section [{section}]")

    raw = key_match.group(1).strip()
    if raw.startswith('"') and raw.endswith('"'):
        return raw[1:-1]
    return raw


def extract_network_identity(network_registry_text: str, network_key: str) -> NetworkIdentity:
    section = f"canonical_networks.{network_key}"
    network_id = extract_toml_section_key(network_registry_text, section, "network_id")
    chain_id = extract_toml_section_key(network_registry_text, section, "chain_id")
    return NetworkIdentity(network_key=network_key, network_id=str(network_id), chain_id=str(chain_id))


def detect_git_commit(repo_root: Path) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo_root,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return result.stdout.strip()


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def default_target_label() -> str:
    machine = platform.machine().lower()
    if machine in {"x86_64", "amd64"}:
        machine = "amd64"
    elif machine in {"aarch64", "arm64"}:
        machine = "arm64"
    system = platform.system().lower()
    return f"{system}-{machine}"


def write_compatibility_toml(path: Path, identity: NetworkIdentity, tracks: dict[str, int], crypto_profile: str) -> None:
    content = (
        "# Repository-governed binary compatibility contract.\n"
        "[compatibility]\n"
        f"release_network = \"{identity.network_key}\"\n"
        f"network_id = [\"{identity.network_id}\"]\n"
        f"chain_id = [\"{identity.chain_id}\"]\n"
        f"crypto_profile = \"{crypto_profile}\"\n"
        f"manifest_schema = {tracks['manifest_schema']}\n"
        f"certificate_schema = {tracks['certificate_schema']}\n"
        f"native_token_policy_schema = {tracks['native_token_policy_schema']}\n"
    )
    path.write_text(content, encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Prepare repository release directory under releases/v<version>.")
    parser.add_argument("--repo-root", default=".", help="Repository root path (default: current directory).")
    parser.add_argument("--binary", default="target/release/aoxc", help="Source binary path.")
    parser.add_argument("--network", default="mainnet", help="Network key from canonical_networks in network-registry.")
    parser.add_argument("--target-label", default=default_target_label(), help="Artifact target label under binaries/.")
    parser.add_argument("--release-line", default="AOXC-Q-v0.2.0", help="Human-facing release line label.")
    parser.add_argument("--crypto-profile", default="aoxcq-v1", help="Compatibility crypto profile.")
    parser.add_argument("--allow-existing", action="store_true", help="Allow writing into existing version directory.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()

    cargo_text = read_text(repo_root / "Cargo.toml")
    policy_text = read_text(repo_root / "configs/version-policy.toml")
    network_registry_text = read_text(repo_root / "configs/registry/network-registry.toml")

    cargo_version = extract_toml_section_key(cargo_text, "workspace.package", "version")
    policy_version = extract_toml_section_key(policy_text, "workspace", "current")
    if cargo_version != policy_version:
        raise ValueError(
            "Version mismatch: Cargo.toml workspace.package.version "
            f"({cargo_version}) != configs/version-policy.toml workspace.current ({policy_version})"
        )
    release_version = cargo_version

    tracks = {
        "manifest_schema": int(extract_toml_section_key(policy_text, "tracks", "manifest_schema")),
        "certificate_schema": int(extract_toml_section_key(policy_text, "tracks", "certificate_schema")),
        "native_token_policy_schema": int(extract_toml_section_key(policy_text, "tracks", "native_token_policy_schema")),
    }
    identity = extract_network_identity(network_registry_text, args.network)

    release_dir = repo_root / "releases" / f"v{release_version}"
    if release_dir.exists() and not args.allow_existing:
        raise FileExistsError(
            f"Release directory already exists: {release_dir}. Use --allow-existing to update metadata intentionally."
        )

    source_binary = (repo_root / args.binary).resolve() if not Path(args.binary).is_absolute() else Path(args.binary)
    if not source_binary.exists() or not source_binary.is_file():
        raise FileNotFoundError(f"Source binary missing: {source_binary}")

    binaries_dir = release_dir / "binaries" / args.target_label
    signatures_dir = release_dir / "signatures"
    binaries_dir.mkdir(parents=True, exist_ok=True)
    signatures_dir.mkdir(parents=True, exist_ok=True)

    destination_binary = binaries_dir / "aoxc"
    shutil.copy2(source_binary, destination_binary)
    destination_binary.chmod(0o755)

    checksum_value = file_sha256(destination_binary)
    checksum_file = release_dir / "checksums.sha256"
    checksum_file.write_text(f"{checksum_value}  binaries/{args.target_label}/aoxc\n", encoding="utf-8")

    compatibility_file = release_dir / "compatibility.toml"
    write_compatibility_toml(compatibility_file, identity, tracks, args.crypto_profile)

    manifest = {
        "release_version": release_version,
        "release_line": args.release_line,
        "published_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "git": {"tag": f"v{release_version}", "commit": detect_git_commit(repo_root)},
        "artifacts": [
            {
                "name": f"aoxc-{args.target_label}",
                "path": f"binaries/{args.target_label}/aoxc",
                "sha256": checksum_value,
                "signature": f"signatures/aoxc-{args.target_label}.sig",
            }
        ],
        "compatibility": {
            "network": args.network,
            "network_id": [identity.network_id],
            "chain_id": [identity.chain_id],
            "crypto_profile": args.crypto_profile,
            "manifest_schema": tracks["manifest_schema"],
            "certificate_schema": tracks["certificate_schema"],
            "native_token_policy_schema": tracks["native_token_policy_schema"],
        },
    }

    manifest_file = release_dir / "manifest.json"
    manifest_file.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    print(f"Prepared repository release directory: {release_dir}")
    print(f"- binary: {destination_binary.relative_to(repo_root)}")
    print(f"- manifest: {manifest_file.relative_to(repo_root)}")
    print(f"- checksums: {checksum_file.relative_to(repo_root)}")
    print(f"- compatibility: {compatibility_file.relative_to(repo_root)}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"[prepare_repo_release][error] {exc}", file=sys.stderr)
        raise SystemExit(2)
