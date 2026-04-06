#!/usr/bin/env python3
"""Fail-closed identity and genesis consistency gate for AOXC environments."""

from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Tuple

try:
    import tomllib  # Python 3.11+
except ModuleNotFoundError:  # pragma: no cover - fallback for older runtimes
    import tomli as tomllib  # type: ignore[no-redef]


REPO_ROOT = Path(__file__).resolve().parents[2]
REGISTRY_PATH = REPO_ROOT / "configs/registry/network-registry.toml"


@dataclass(frozen=True)
class IdentityTuple:
    chain_id: int
    network_id: str
    network_serial: str


class GateError(RuntimeError):
    pass


def load_toml(path: Path) -> dict:
    with path.open("rb") as f:
        return tomllib.load(f)


def load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def canonical_map(registry: dict) -> Dict[str, IdentityTuple]:
    result: Dict[str, IdentityTuple] = {}
    for env, obj in registry.get("canonical_networks", {}).items():
        result[env] = IdentityTuple(
            chain_id=int(obj["chain_id"]),
            network_id=str(obj["network_id"]),
            network_serial=str(obj["network_serial"]),
        )
    return result


def read_identity_from_release_policy(path: Path) -> IdentityTuple:
    policy = load_toml(path)
    identity = policy["identity"]
    return IdentityTuple(
        chain_id=int(identity["chain_id"]),
        network_id=str(identity["network_id"]),
        network_serial=str(identity["network_serial"]),
    )


def read_identity_from_profile(path: Path) -> IdentityTuple:
    profile = load_toml(path)
    identity = profile["identity"]
    return IdentityTuple(
        chain_id=int(identity["chain_id"]),
        network_id=str(identity["network_id"]),
        network_serial=str(identity["network_serial"]),
    )


def read_identity_from_genesis(path: Path) -> IdentityTuple:
    genesis = load_json(path)
    identity = genesis["identity"]
    return IdentityTuple(
        chain_id=int(identity["chain_id"]),
        network_id=str(identity["network_id"]),
        network_serial=str(identity["network_serial"]),
    )


def read_genesis_sha256(path: Path) -> str:
    content = path.read_bytes()
    return hashlib.sha256(content).hexdigest()


def read_hash_file(path: Path) -> str:
    line = path.read_text(encoding="utf-8").strip().split()[0]
    if len(line) != 64:
        raise GateError(f"invalid hash format in {path}")
    return line.lower()


def assert_no_identity_overrides(path: Path) -> None:
    policy = load_toml(path)
    promotion = policy.get("promotion", {})
    checks = {
        "allow_chain_id_override": False,
        "allow_network_id_override": False,
        "allow_manifest_identity_override": False,
    }
    for key, expected in checks.items():
        actual = promotion.get(key)
        if actual is not expected:
            raise GateError(
                f"{path}: expected {key}={expected!r} but found {actual!r}"
            )


def validate_environment(env: str, expected: IdentityTuple) -> None:
    env_root = REPO_ROOT / "configs" / "environments" / env
    release_policy_path = env_root / "release-policy.toml"
    profile_path = env_root / "profile.toml"
    genesis_path = env_root / "genesis.v1.json"
    genesis_hash_path = env_root / "genesis.v1.sha256"

    if not release_policy_path.exists():
        raise GateError(f"missing release policy: {release_policy_path}")
    if not profile_path.exists():
        raise GateError(f"missing profile: {profile_path}")
    if not genesis_path.exists():
        raise GateError(f"missing genesis: {genesis_path}")
    if not genesis_hash_path.exists():
        raise GateError(f"missing genesis hash: {genesis_hash_path}")

    sources: List[Tuple[str, IdentityTuple]] = [
        ("registry", expected),
        ("release-policy", read_identity_from_release_policy(release_policy_path)),
        ("profile", read_identity_from_profile(profile_path)),
        ("genesis", read_identity_from_genesis(genesis_path)),
    ]

    baseline = sources[0][1]
    for source_name, current in sources[1:]:
        if current != baseline:
            raise GateError(
                f"identity mismatch for {env}: {source_name}={current} expected={baseline}"
            )

    assert_no_identity_overrides(release_policy_path)

    computed = read_genesis_sha256(genesis_path)
    declared = read_hash_file(genesis_hash_path)
    if computed != declared:
        raise GateError(
            f"genesis hash mismatch for {env}: declared={declared} computed={computed}"
        )



def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Fail-closed identity consistency gate for AOXC environments"
    )
    parser.add_argument(
        "--env",
        action="append",
        help="Specific environment(s) to validate. Can be supplied multiple times.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    registry = load_toml(REGISTRY_PATH)
    canonical = canonical_map(registry)
    selected: Iterable[str] = args.env or [
        "mainnet",
        "testnet",
        "devnet",
        "validation",
        "localnet",
    ]

    for env in selected:
        if env not in canonical:
            raise GateError(f"{env!r} is not defined in canonical_networks")
        validate_environment(env, canonical[env])
        print(f"OK: {env} identity/genesis tuple is consistent and fail-closed")

    print("OK: network identity gate passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except GateError as exc:
        print(f"ERROR: {exc}")
        raise SystemExit(2)
