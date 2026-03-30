#!/usr/bin/env python3
"""Validate one AOXC runtime bundle for the active single-system network kind."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import sys

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover - python < 3.11 fallback
    import tomli as tomllib  # type: ignore[no-redef]

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
CONFIG_ROOT = REPO_ROOT / "configs"
ENV_ROOT = CONFIG_ROOT / "environments"

ENV_SPECS = {
    "mainnet": ENV_ROOT / "mainnet",
    "testnet": ENV_ROOT / "testnet",
    "devnet": ENV_ROOT / "devnet",
    "localnet": ENV_ROOT / "localnet",
    "validation": ENV_ROOT / "validation",
}

REQUIRED_FILES = (
    "manifest.v1.json",
    "genesis.v1.json",
    "genesis.v1.sha256",
    "validators.json",
    "bootnodes.json",
    "certificate.json",
    "profile.toml",
    "release-policy.toml",
)


def fail(message: str) -> None:
    print(f"[bundle-check][error] {message}", file=sys.stderr)
    raise SystemExit(1)


def read_json(path: pathlib.Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        fail(f"Cannot parse JSON: {path} ({exc})")


def read_toml(path: pathlib.Path) -> dict:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        fail(f"Cannot parse TOML: {path} ({exc})")


def expected_sha_line(genesis_path: pathlib.Path) -> str:
    digest = hashlib.sha256(genesis_path.read_bytes()).hexdigest()
    return f"{digest}  genesis.v1.json"


def canonical_identity(root: pathlib.Path) -> tuple[str, str, int]:
    manifest = read_json(root / "manifest.v1.json")
    genesis = read_json(root / "genesis.v1.json")
    validators = read_json(root / "validators.json")
    bootnodes = read_json(root / "bootnodes.json")
    certificate = read_json(root / "certificate.json")
    profile = read_toml(root / "profile.toml")
    release_policy = read_toml(root / "release-policy.toml")

    identities = {
        "manifest": (manifest["environment"], manifest["identity"]["network_id"], int(manifest["identity"]["chain_id"])),
        "genesis": (genesis["environment"], genesis["identity"]["network_id"], int(genesis["identity"]["chain_id"])),
        "validators": (validators["environment"], validators["identity"]["network_id"], int(validators["identity"]["chain_id"])),
        "bootnodes": (bootnodes["environment"], bootnodes["identity"]["network_id"], int(bootnodes["identity"]["chain_id"])),
        "certificate": (certificate["environment"], certificate["identity"]["network_id"], int(certificate["identity"]["chain_id"])),
        "profile": (
            profile["profile"]["environment"],
            profile["identity"]["network_id"],
            int(profile["identity"]["chain_id"]),
        ),
        "release_policy": (
            release_policy["policy"]["environment"],
            release_policy["identity"]["network_id"],
            int(release_policy["identity"]["chain_id"]),
        ),
    }

    expected = identities["manifest"]
    for source, tuple_value in identities.items():
        if tuple_value != expected:
            fail(
                f"Identity mismatch in {root} between manifest and {source}: "
                f"manifest={expected} {source}={tuple_value}"
            )

    return expected


def validate_environment(kind: str, root: pathlib.Path) -> tuple[str, str, int]:
    if not root.is_dir():
        fail(f"Missing environment directory: {root}")

    for filename in REQUIRED_FILES:
        path = root / filename
        if not path.is_file():
            fail(f"Missing required file for {kind}: {path}")

    identity = canonical_identity(root)
    if identity[0] != kind:
        fail(f"Environment mismatch for {kind}: manifest declares {identity[0]}")

    sha_path = root / "genesis.v1.sha256"
    sha_line = sha_path.read_text(encoding="utf-8").strip()
    expected_line = expected_sha_line(root / "genesis.v1.json")
    if sha_line != expected_line:
        fail(f"Invalid genesis checksum in {sha_path}. Expected '{expected_line}'")

    print(f"[bundle-check][ok] {kind}: identity and checksum consistent")
    return identity


def validate_aoxhub_profile_if_available(kind: str, expected: tuple[str, str, int]) -> None:
    profile_path = CONFIG_ROOT / "aoxhub" / f"{kind}.toml"
    if not profile_path.is_file():
        print(f"[bundle-check][info] no aoxhub mapping for {kind}; skipped")
        return

    data = read_toml(profile_path)
    declared_env = data["hub"]["environment"]
    declared_network = data["identity"]["network_id"]
    declared_chain = int(data["identity"]["chain_id"])

    if (declared_env, declared_network, declared_chain) != expected:
        fail(
            f"aoxhub identity mismatch in {profile_path}: "
            f"declared={(declared_env, declared_network, declared_chain)} expected={expected}"
        )

    env_root = f"configs/environments/{kind}"
    if data["paths"]["environment_root"] != env_root:
        fail(f"aoxhub bundle root mismatch in {profile_path}: {data['paths']['environment_root']}")

    print(f"[bundle-check][ok] aoxhub {kind}: mapping aligned")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--network-kind",
        default=os.environ.get("AOXC_NETWORK_KIND", "mainnet"),
        choices=sorted(ENV_SPECS.keys()),
        help="Active single-system network kind.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    kind = args.network_kind
    root = ENV_SPECS[kind]

    print(f"[bundle-check] validating active bundle for AOXC_NETWORK_KIND={kind}")
    expected = validate_environment(kind, root)
    validate_aoxhub_profile_if_available(kind, expected)
    print("[bundle-check] all checks passed")


if __name__ == "__main__":
    main()
