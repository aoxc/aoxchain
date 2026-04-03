#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

PROFILE="${AOXC_BOOTSTRAP_PROFILE:-localnet}"
VALIDATOR_NAME="${AOXC_VALIDATOR_NAME:-validator-01}"
VALIDATOR_PASSWORD="${AOXC_VALIDATOR_PASSWORD:-ChangeMe#2026}"

require_non_empty "${PROFILE}" "AOXC_BOOTSTRAP_PROFILE"
require_non_empty "${VALIDATOR_NAME}" "AOXC_VALIDATOR_NAME"
require_non_empty "${VALIDATOR_PASSWORD}" "AOXC_VALIDATOR_PASSWORD"

run_make_target chain-init \
  AOXC_BOOTSTRAP_PROFILE="${PROFILE}" \
  AOXC_VALIDATOR_NAME="${VALIDATOR_NAME}" \
  AOXC_VALIDATOR_PASSWORD="${VALIDATOR_PASSWORD}"
