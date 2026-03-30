#!/usr/bin/env python3
"""Validate AOXC environment bundles for single-system runtime compatibility."""

from __future__ import annotations

import hashlib
import json
import pathlib
import sys
try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover - python < 3.11 fallback
    import tomli as tomllib  # type: ignore[no-redef]

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
CONFIG_ROOT = REPO_ROOT / "configs"
ENV_ROOT = CONFIG_ROOT / "environments"
NETWORK_KINDS = ("mainnet", "testnet", "devnet")
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
    except Exception as exc:  # pragma: no cover - guardrail path
        fail(f"Cannot parse JSON: {path} ({exc})")


def read_toml(path: pathlib.Path) -> dict:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover - guardrail path
        fail(f"Cannot parse TOML: {path} ({exc})")


def expected_sha_line(genesis_path: pathlib.Path) -> str:
    digest = hashlib.sha256(genesis_path.read_bytes()).hexdigest()
    return f"{digest}  genesis.v1.json"


def validate_environment(kind: str) -> None:
    root = ENV_ROOT / kind
    if not root.is_dir():
        fail(f"Missing environment directory: {root}")

    for filename in REQUIRED_FILES:
        path = root / filename
        if not path.is_file():
            fail(f"Missing required file for {kind}: {path}")

    manifest = read_json(root / "manifest.v1.json")
    genesis = read_json(root / "genesis.v1.json")
    validators = read_json(root / "validators.json")
    bootnodes = read_json(root / "bootnodes.json")
    certificate = read_json(root / "certificate.json")
    profile = read_toml(root / "profile.toml")
    release_policy = read_toml(root / "release-policy.toml")

    identities = {
        "manifest": (manifest["environment"], manifest["identity"]["network_id"], manifest["identity"]["chain_id"]),
        "genesis": (genesis["environment"], genesis["identity"]["network_id"], genesis["identity"]["chain_id"]),
        "validators": (validators["environment"], validators["identity"]["network_id"], validators["identity"]["chain_id"]),
        "bootnodes": (bootnodes["environment"], bootnodes["identity"]["network_id"], bootnodes["identity"]["chain_id"]),
        "certificate": (certificate["environment"], certificate["identity"]["network_id"], certificate["identity"]["chain_id"]),
        "profile": (
            profile["profile"]["environment"],
            profile["identity"]["network_id"],
            profile["identity"]["chain_id"],
        ),
        "release_policy": (
            release_policy["policy"]["environment"],
            release_policy["identity"]["network_id"],
            release_policy["identity"]["chain_id"],
        ),
    }

    expected = identities["manifest"]
    if expected[0] != kind:
        fail(f"Manifest environment does not match folder for {kind}: {expected[0]}")

    for source, tuple_value in identities.items():
        if tuple_value != expected:
            fail(
                f"Identity mismatch for {kind} between manifest and {source}: "
                f"manifest={expected} {source}={tuple_value}"
            )

    sha_path = root / "genesis.v1.sha256"
    sha_line = sha_path.read_text(encoding="utf-8").strip()
    expected_line = expected_sha_line(root / "genesis.v1.json")
    if sha_line != expected_line:
        fail(f"Invalid genesis checksum in {sha_path}. Expected '{expected_line}'")

    print(f"[bundle-check][ok] {kind}: identity and checksum consistent")


def validate_aoxhub_profiles() -> None:
    profile_root = CONFIG_ROOT / "aoxhub"
    for kind in ("mainnet", "testnet"):
        path = profile_root / f"{kind}.toml"
        data = read_toml(path)
        declared = data["hub"]["environment"]
        if declared != kind:
            fail(f"aoxhub profile env mismatch in {path}: {declared}")

        env_root = f"configs/environments/{kind}"
        if data["paths"]["environment_root"] != env_root:
            fail(f"aoxhub bundle root mismatch in {path}: {data['paths']['environment_root']}")

    print("[bundle-check][ok] aoxhub profiles are aligned")


def main() -> None:
    print("[bundle-check] validating canonical environment bundles")
    for kind in NETWORK_KINDS:
        validate_environment(kind)

    validate_aoxhub_profiles()
    print("[bundle-check] all checks passed")


if __name__ == "__main__":
    main()
