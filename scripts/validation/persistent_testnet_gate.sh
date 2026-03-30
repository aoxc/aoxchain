#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
# Persistent testnet minimum gate (P0 + key P1 controls)
# -----------------------------------------------------------------------------
#
# Purpose:
#   Enforce a minimum persistent testnet gate for the canonical AOXC testnet
#   environment bundle.
#
# Scope:
#   - Validate presence of required environment files
#   - Validate minimum validator and bootnode counts
#   - Validate published network metadata fields
#   - Validate fixed manifest identity fields
#
# Exit Codes:
#   0  Successful completion
#   1  Gate failure
#   2  Invalid runtime dependency or configuration
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
readonly TESTNET_DIR="${ROOT_DIR}/configs/environments/testnet"

log_pass() {
  printf '[PASS] %s\n' "$*"
}

log_fail() {
  printf '[FAIL] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="${2:-1}"
  log_fail "${message}"
  exit "${exit_code}"
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 2
}

require_file() {
  local file_path="$1"
  [[ -f "${file_path}" ]] || die "Missing required file: ${file_path}" 1
}

validate_required_files() {
  local required_files=(
    "manifest.v1.json"
    "profile.toml"
    "release-policy.toml"
    "genesis.v1.json"
    "genesis.v1.sha256"
    "validators.json"
    "bootnodes.json"
    "network-metadata.json"
  )
  local file_name=''

  for file_name in "${required_files[@]}"; do
    require_file "${TESTNET_DIR}/${file_name}"
  done

  log_pass "Required environment files are present."
}

validate_validator_count() {
  local validator_count=''

  validator_count="$(jq -r '
    if type == "object" and (.validators | type == "array") then
      (.validators | length)
    else
      error("validators.json must contain a top-level validators array")
    end
  ' "${TESTNET_DIR}/validators.json")"

  [[ "${validator_count}" =~ ^[0-9]+$ ]] || die "Unable to resolve validator count." 1
  (( validator_count >= 3 )) || die "validators.json must contain at least 3 validators." 1

  log_pass "Validator count is >= 3."
}

validate_bootnode_count() {
  local bootnode_count=''

  bootnode_count="$(jq -r '
    if type == "object" and (.bootnodes | type == "array") then
      (.bootnodes | length)
    else
      error("bootnodes.json must contain a top-level bootnodes array")
    end
  ' "${TESTNET_DIR}/bootnodes.json")"

  [[ "${bootnode_count}" =~ ^[0-9]+$ ]] || die "Unable to resolve bootnode count." 1
  (( bootnode_count >= 1 )) || die "bootnodes.json must contain at least 1 bootnode." 1

  log_pass "Bootnode count is >= 1."
}

validate_network_metadata() {
  jq -e '
    type == "object"
    and (.rpc | type == "object")
    and (.rpc.primary | type == "string" and length > 0)
    and (.chain_id | type == "number")
    and (.network_id | type == "string" and length > 0)
    and (.native_symbol | type == "string" and length > 0)
  ' "${TESTNET_DIR}/network-metadata.json" >/dev/null \
    || die "network-metadata.json must publish rpc + chain metadata." 1

  log_pass "Network metadata includes chain and RPC fields."
}

validate_manifest_identity() {
  jq -e '
    type == "object"
    and (.identity | type == "object")
    and (.identity.chain_id == 2626010001)
    and (.identity.network_id == "aoxc-testnet-2626-002")
  ' "${TESTNET_DIR}/manifest.v1.json" >/dev/null \
    || die "manifest.v1.json identity mismatch." 1

  log_pass "Manifest identity is fixed and correct."
}

main() {
  require_command jq
  [[ -d "${TESTNET_DIR}" ]] || die "Testnet environment directory does not exist: ${TESTNET_DIR}" 2

  validate_required_files
  validate_validator_count
  validate_bootnode_count
  validate_network_metadata
  validate_manifest_identity

  printf '[OK] persistent testnet gate passed\n'
}

main "$@"
