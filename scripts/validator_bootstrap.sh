#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

usage() {
  cat <<'OUT'
Usage: scripts/validator_bootstrap.sh [--dry-run] [--verbose] [--print-env]
Adds validator material to genesis via `make chain-add-validator`.
OUT
  print_common_flag_help
}

args=("$@")
if [[ " ${args[*]} " == *" --help "* ]]; then usage; exit 0; fi
parse_common_flags args
PRINT_ENV=0
for arg in "${args[@]}"; do
  case "${arg}" in
    --print-env) PRINT_ENV=1 ;;
    *) die "Unknown argument: ${arg}" 2 ;;
  esac
done

VALIDATOR_ID="${AOXC_VALIDATOR_ID:-aoxc-val-custom-001}"
CONSENSUS_KEY="${AOXC_CONSENSUS_PUBLIC_KEY:-replace_with_consensus_public_key_hex}"
NETWORK_KEY="${AOXC_NETWORK_PUBLIC_KEY:-replace_with_network_public_key_hex}"
BALANCE="${AOXC_VALIDATOR_BALANCE:-50000000}"

require_non_empty "${VALIDATOR_ID}" "AOXC_VALIDATOR_ID"
require_non_placeholder_key "${CONSENSUS_KEY}" "AOXC_CONSENSUS_PUBLIC_KEY"
require_non_placeholder_key "${NETWORK_KEY}" "AOXC_NETWORK_PUBLIC_KEY"
require_positive_uint "${BALANCE}" "AOXC_VALIDATOR_BALANCE"

if [[ "${PRINT_ENV}" == "1" ]]; then
  printf 'AOXC_VALIDATOR_ID=%s\nAOXC_VALIDATOR_BALANCE=%s\n' "${VALIDATOR_ID}" "${BALANCE}"
fi

run_make_target chain-add-validator \
  AOXC_VALIDATOR_ID="${VALIDATOR_ID}" \
  AOXC_CONSENSUS_PUBLIC_KEY="${CONSENSUS_KEY}" \
  AOXC_NETWORK_PUBLIC_KEY="${NETWORK_KEY}" \
  AOXC_VALIDATOR_BALANCE="${BALANCE}"
