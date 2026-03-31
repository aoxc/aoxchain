#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
TESTNET_ROOT="${REPO_ROOT}/configs/environments/testnet"

required_files=(
  manifest.v1.json
  genesis.v1.json
  genesis.v1.sha256
  validators.json
  bootnodes.json
  certificate.json
  profile.toml
  release-policy.toml
  network-metadata.json
)

echo "[testnet-gate] starting persistent testnet readiness gate"

for rel in "${required_files[@]}"; do
  if [[ ! -f "${TESTNET_ROOT}/${rel}" ]]; then
    echo "[testnet-gate][error] missing required file: ${TESTNET_ROOT}/${rel}" >&2
    exit 1
  fi
done

echo "[testnet-gate] running cross-environment bundle consistency validator"
python3 "${REPO_ROOT}/scripts/validate_environment_bundle.py"

echo "[testnet-gate] validating testnet runtime source bundle via Make surface"
make -C "${REPO_ROOT}" --no-print-directory runtime-source-check AOXC_NETWORK_KIND=testnet

echo "[testnet-gate] validating testnet metadata identity alignment"
python3 - "${TESTNET_ROOT}" <<'PY'
import json
import hashlib
import pathlib
import re
import sys
from datetime import datetime

root = pathlib.Path(sys.argv[1])
manifest = json.loads((root / "manifest.v1.json").read_text(encoding="utf-8"))
metadata = json.loads((root / "network-metadata.json").read_text(encoding="utf-8"))
validators = json.loads((root / "validators.json").read_text(encoding="utf-8"))
bootnodes = json.loads((root / "bootnodes.json").read_text(encoding="utf-8"))
certificate = json.loads((root / "certificate.json").read_text(encoding="utf-8"))

required_top_level = ["network_name", "network_id", "chain_id", "rpc", "public_endpoints"]
for key in required_top_level:
    if key not in metadata:
        raise SystemExit(f"[testnet-gate][error] network-metadata.json missing key: {key}")

for key in ("primary", "secondary", "ws"):
    if key not in metadata["rpc"]:
        raise SystemExit(f"[testnet-gate][error] network-metadata.json missing rpc.{key}")

for key in ("faucet", "explorer", "status"):
    if key not in metadata["public_endpoints"]:
        raise SystemExit(f"[testnet-gate][error] network-metadata.json missing public_endpoints.{key}")

manifest_identity = manifest["identity"]

if metadata.get("network_id") != manifest_identity.get("network_id"):
    raise SystemExit(
        "[testnet-gate][error] metadata network_id mismatch: "
        f"{metadata.get('network_id')} != {manifest_identity.get('network_id')}"
    )

if metadata.get("chain_id") != manifest_identity.get("chain_id"):
    raise SystemExit(
        "[testnet-gate][error] metadata chain_id mismatch: "
        f"{metadata.get('chain_id')} != {manifest_identity.get('chain_id')}"
    )

validator_entries = validators.get("validators")
if not isinstance(validator_entries, list) or len(validator_entries) < 3:
    raise SystemExit("[testnet-gate][error] validators.json must define at least 3 validators")

bootnode_entries = bootnodes.get("bootnodes")
if not isinstance(bootnode_entries, list) or len(bootnode_entries) < 2:
    raise SystemExit("[testnet-gate][error] bootnodes.json must define at least 2 bootnodes")

validator_ids = [entry.get("validator_id") for entry in validator_entries]
if len(set(validator_ids)) != len(validator_ids):
    raise SystemExit("[testnet-gate][error] validators.json contains duplicate validator_id values")

bootnode_ids = [entry.get("node_id") for entry in bootnode_entries]
if len(set(bootnode_ids)) != len(bootnode_ids):
    raise SystemExit("[testnet-gate][error] bootnodes.json contains duplicate node_id values")

def contains_placeholder(value: str) -> bool:
    normalized = value.upper()
    return "REPLACE_WITH" in normalized or "PENDING_REAL_VALUE" in normalized

def require_hex(value: str, expected_len: int, field_path: str) -> None:
    if not re.fullmatch(r"[0-9a-fA-F]+", value):
        raise SystemExit(f"[testnet-gate][error] {field_path} must be hex")
    if len(value) != expected_len:
        raise SystemExit(
            f"[testnet-gate][error] {field_path} must be {expected_len} hex chars, got {len(value)}"
        )

for entry in validator_entries:
    for field in (
        "consensus_public_key",
        "consensus_key_fingerprint",
        "network_public_key",
        "network_key_fingerprint",
    ):
        raw = entry.get(field)
        if not isinstance(raw, str) or not raw.strip():
            raise SystemExit(f"[testnet-gate][error] validators.json missing non-empty {field}")
        if contains_placeholder(raw):
            raise SystemExit(f"[testnet-gate][error] validators.json contains placeholder in {field}")
        require_hex(raw, 64, f"validators.json[{entry.get('validator_id')}].{field}")

    expected_consensus_fingerprint = hashlib.sha256(
        bytes.fromhex(entry["consensus_public_key"])
    ).hexdigest()
    if entry["consensus_key_fingerprint"].lower() != expected_consensus_fingerprint:
        raise SystemExit(
            "[testnet-gate][error] validators.json consensus_key_fingerprint "
            f"does not match sha256(consensus_public_key) for {entry.get('validator_id')}"
        )

    expected_network_fingerprint = hashlib.sha256(
        bytes.fromhex(entry["network_public_key"])
    ).hexdigest()
    if entry["network_key_fingerprint"].lower() != expected_network_fingerprint:
        raise SystemExit(
            "[testnet-gate][error] validators.json network_key_fingerprint "
            f"does not match sha256(network_public_key) for {entry.get('validator_id')}"
        )

for entry in bootnode_entries:
    for field in ("transport_public_key", "transport_key_fingerprint"):
        raw = entry.get(field)
        if not isinstance(raw, str) or not raw.strip():
            raise SystemExit(f"[testnet-gate][error] bootnodes.json missing non-empty {field}")
        if contains_placeholder(raw):
            raise SystemExit(f"[testnet-gate][error] bootnodes.json contains placeholder in {field}")
        require_hex(raw, 64, f"bootnodes.json[{entry.get('node_id')}].{field}")

    expected_transport_fingerprint = hashlib.sha256(
        bytes.fromhex(entry["transport_public_key"])
    ).hexdigest()
    if entry["transport_key_fingerprint"].lower() != expected_transport_fingerprint:
        raise SystemExit(
            "[testnet-gate][error] bootnodes.json transport_key_fingerprint "
            f"does not match sha256(transport_public_key) for {entry.get('node_id')}"
        )

cert = certificate.get("certificate", {})
for field in ("issued_at", "fingerprint_sha256", "signature"):
    raw = cert.get(field)
    if field == "issued_at" and raw is None:
        raise SystemExit("[testnet-gate][error] certificate.json missing certificate.issued_at")
    if field != "issued_at" and (not isinstance(raw, str) or not raw.strip()):
        raise SystemExit(f"[testnet-gate][error] certificate.json missing non-empty certificate.{field}")
    if isinstance(raw, str) and contains_placeholder(raw):
        raise SystemExit(f"[testnet-gate][error] certificate.json contains placeholder in certificate.{field}")

issued_at_raw = cert.get("issued_at")
if issued_at_raw is not None:
    if not isinstance(issued_at_raw, str):
        raise SystemExit("[testnet-gate][error] certificate.issued_at must be an RFC3339 string")
    try:
        datetime.fromisoformat(issued_at_raw.replace("Z", "+00:00"))
    except ValueError as exc:
        raise SystemExit(f"[testnet-gate][error] certificate.issued_at is not valid RFC3339: {exc}")

require_hex(cert["fingerprint_sha256"], 64, "certificate.fingerprint_sha256")
require_hex(cert["signature"], 128, "certificate.signature")

print("[testnet-gate][ok] metadata identity alignment is valid")
PY

echo "[testnet-gate] all checks passed"
