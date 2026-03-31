#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

VALIDATOR_ID="${AOXC_VALIDATOR_ID:-aoxc-val-custom-001}"
CONSENSUS_KEY="${AOXC_CONSENSUS_PUBLIC_KEY:-replace_with_consensus_public_key_hex}"
NETWORK_KEY="${AOXC_NETWORK_PUBLIC_KEY:-replace_with_network_public_key_hex}"
BALANCE="${AOXC_VALIDATOR_BALANCE:-50000000}"

make --no-print-directory chain-add-validator \
  AOXC_VALIDATOR_ID="${VALIDATOR_ID}" \
  AOXC_CONSENSUS_PUBLIC_KEY="${CONSENSUS_KEY}" \
  AOXC_NETWORK_PUBLIC_KEY="${NETWORK_KEY}" \
  AOXC_VALIDATOR_BALANCE="${BALANCE}"
