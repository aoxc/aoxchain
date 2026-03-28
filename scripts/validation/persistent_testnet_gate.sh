#!/usr/bin/env bash
# AOXC MIT License
# Persistent testnet minimum gate (P0 + key P1 controls)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TESTNET_DIR="${ROOT_DIR}/configs/environments/testnet"

pass() { echo "[PASS] $1"; }
fail() { echo "[FAIL] $1" >&2; exit 1; }

required_files=(
  "manifest.v1.json"
  "profile.toml"
  "release-policy.toml"
  "genesis.v1.json"
  "genesis.v1.sha256"
  "validators.json"
  "bootnodes.json"
  "network-metadata.json"
)

for file in "${required_files[@]}"; do
  [[ -f "${TESTNET_DIR}/${file}" ]] || fail "missing required file: ${file}"
done
pass "required environment files present"

validator_count="$(jq '.validators | length' "${TESTNET_DIR}/validators.json")"
[[ "${validator_count}" -ge 3 ]] || fail "validators.json must contain at least 3 validators"
pass "validator count >= 3"

bootnode_count="$(jq '.bootnodes | length' "${TESTNET_DIR}/bootnodes.json")"
[[ "${bootnode_count}" -ge 1 ]] || fail "bootnodes.json must contain at least 1 bootnode"
pass "bootnode count >= 1"

jq -e '.rpc.primary and .chain_id and .network_id and .native_symbol' "${TESTNET_DIR}/network-metadata.json" >/dev/null \
  || fail "network-metadata.json must publish rpc + chain metadata"
pass "network metadata includes chain + rpc fields"

jq -e '.identity.chain_id == 2626010001 and .identity.network_id == "aoxc-testnet-2626-002"' "${TESTNET_DIR}/manifest.v1.json" >/dev/null \
  || fail "manifest identity mismatch"
pass "manifest identity fixed"

echo "[OK] persistent testnet gate passed"
