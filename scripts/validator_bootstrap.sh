#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

VALIDATOR_ID="${AOXC_VALIDATOR_ID:-aoxc-val-custom-001}"
CONSENSUS_KEY="${AOXC_CONSENSUS_PUBLIC_KEY:-replace_with_consensus_public_key_hex}"
NETWORK_KEY="${AOXC_NETWORK_PUBLIC_KEY:-replace_with_network_public_key_hex}"
BALANCE="${AOXC_VALIDATOR_BALANCE:-50000000}"

require_non_empty "${VALIDATOR_ID}" "AOXC_VALIDATOR_ID"
require_non_placeholder_key "${CONSENSUS_KEY}" "AOXC_CONSENSUS_PUBLIC_KEY"
require_non_placeholder_key "${NETWORK_KEY}" "AOXC_NETWORK_PUBLIC_KEY"
require_uint "${BALANCE}" "AOXC_VALIDATOR_BALANCE"

run_make_target chain-add-validator \
  AOXC_VALIDATOR_ID="${VALIDATOR_ID}" \
  AOXC_CONSENSUS_PUBLIC_KEY="${CONSENSUS_KEY}" \
  AOXC_NETWORK_PUBLIC_KEY="${NETWORK_KEY}" \
  AOXC_VALIDATOR_BALANCE="${BALANCE}"
