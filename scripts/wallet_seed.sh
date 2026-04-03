#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

ACCOUNT_ID="${AOXC_NEW_ACCOUNT_ID:-AOXC_USER_ALICE}"
BALANCE="${AOXC_NEW_ACCOUNT_BALANCE:-1000000}"
ROLE="${AOXC_NEW_ACCOUNT_ROLE:-user}"

require_non_empty "${ACCOUNT_ID}" "AOXC_NEW_ACCOUNT_ID"
require_uint "${BALANCE}" "AOXC_NEW_ACCOUNT_BALANCE"
require_non_empty "${ROLE}" "AOXC_NEW_ACCOUNT_ROLE"

run_make_target chain-add-account \
  AOXC_NEW_ACCOUNT_ID="${ACCOUNT_ID}" \
  AOXC_NEW_ACCOUNT_BALANCE="${BALANCE}" \
  AOXC_NEW_ACCOUNT_ROLE="${ROLE}"
