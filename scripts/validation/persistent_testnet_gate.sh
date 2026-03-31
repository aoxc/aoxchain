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
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
manifest = json.loads((root / "manifest.v1.json").read_text(encoding="utf-8"))
metadata = json.loads((root / "network-metadata.json").read_text(encoding="utf-8"))

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

print("[testnet-gate][ok] metadata identity alignment is valid")
PY

echo "[testnet-gate] all checks passed"
