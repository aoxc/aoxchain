#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

: "${AOXC_BOOTSTRAP_PROFILE:=localnet}"
: "${AOXC_VALIDATOR_NAME:=validator-01}"
: "${AOXC_VALIDATOR_PASSWORD:=DemoPass#2026}"
: "${AOXC_NEW_ACCOUNT_ID:=AOXC_USER_ALICE}"
: "${AOXC_NEW_ACCOUNT_BALANCE:=1000000}"
: "${AOXC_TRANSFER_TO:=bob}"
: "${AOXC_TRANSFER_AMOUNT:=100}"

require_uint "${AOXC_NEW_ACCOUNT_BALANCE}" "AOXC_NEW_ACCOUNT_BALANCE"
require_uint "${AOXC_TRANSFER_AMOUNT}" "AOXC_TRANSFER_AMOUNT"

"${AOXC_REPO_ROOT}/scripts/preflight_check.sh"
"${AOXC_REPO_ROOT}/scripts/chain_create.sh"
"${AOXC_REPO_ROOT}/scripts/wallet_seed.sh"
"${AOXC_REPO_ROOT}/scripts/network_start.sh"
"${AOXC_REPO_ROOT}/scripts/finality_smoke.sh"
"${AOXC_REPO_ROOT}/scripts/transfer_smoke.sh"

cat <<OUT
AOXChain demo flow completed.
profile=${AOXC_BOOTSTRAP_PROFILE}
seeded_account=${AOXC_NEW_ACCOUNT_ID}
transfer_to=${AOXC_TRANSFER_TO}
transfer_amount=${AOXC_TRANSFER_AMOUNT}
OUT
