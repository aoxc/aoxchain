#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

usage() {
  cat <<'OUT'
Usage: scripts/chain_demo.sh [--dry-run] [--verbose] [--print-env]
Runs end-to-end demo flow (preflight, chain init, account seed, start, smokes).
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

: "${AOXC_BOOTSTRAP_PROFILE:=localnet}"
: "${AOXC_VALIDATOR_NAME:=validator-01}"
: "${AOXC_VALIDATOR_PASSWORD:=DemoPass#2026}"
: "${AOXC_NEW_ACCOUNT_ID:=AOXC_USER_ALICE}"
: "${AOXC_NEW_ACCOUNT_BALANCE:=1000000}"
: "${AOXC_TRANSFER_TO:=bob}"
: "${AOXC_TRANSFER_AMOUNT:=100}"

require_non_empty "${AOXC_BOOTSTRAP_PROFILE}" "AOXC_BOOTSTRAP_PROFILE"
require_non_empty "${AOXC_VALIDATOR_NAME}" "AOXC_VALIDATOR_NAME"
require_non_empty "${AOXC_VALIDATOR_PASSWORD}" "AOXC_VALIDATOR_PASSWORD"
require_non_empty "${AOXC_NEW_ACCOUNT_ID}" "AOXC_NEW_ACCOUNT_ID"
require_non_empty "${AOXC_TRANSFER_TO}" "AOXC_TRANSFER_TO"
require_positive_uint "${AOXC_NEW_ACCOUNT_BALANCE}" "AOXC_NEW_ACCOUNT_BALANCE"
require_positive_uint "${AOXC_TRANSFER_AMOUNT}" "AOXC_TRANSFER_AMOUNT"

if [[ "${PRINT_ENV}" == "1" ]]; then
  printf 'AOXC_BOOTSTRAP_PROFILE=%s\nAOXC_VALIDATOR_NAME=%s\nAOXC_NEW_ACCOUNT_ID=%s\nAOXC_NEW_ACCOUNT_BALANCE=%s\nAOXC_TRANSFER_TO=%s\nAOXC_TRANSFER_AMOUNT=%s\n' \
    "${AOXC_BOOTSTRAP_PROFILE}" "${AOXC_VALIDATOR_NAME}" "${AOXC_NEW_ACCOUNT_ID}" "${AOXC_NEW_ACCOUNT_BALANCE}" "${AOXC_TRANSFER_TO}" "${AOXC_TRANSFER_AMOUNT}"
fi

step_flags=()
[[ "${AOXC_SCRIPT_DRY_RUN}" == "1" ]] && step_flags+=(--dry-run)
[[ "${AOXC_SCRIPT_VERBOSE}" == "1" ]] && step_flags+=(--verbose)

"${AOXC_REPO_ROOT}/scripts/preflight_check.sh" "${step_flags[@]}"
"${AOXC_REPO_ROOT}/scripts/chain_create.sh" "${step_flags[@]}"
"${AOXC_REPO_ROOT}/scripts/wallet_seed.sh" "${step_flags[@]}"
"${AOXC_REPO_ROOT}/scripts/network_start.sh" "${step_flags[@]}"
"${AOXC_REPO_ROOT}/scripts/finality_smoke.sh" "${step_flags[@]}"
"${AOXC_REPO_ROOT}/scripts/transfer_smoke.sh" "${step_flags[@]}"

cat <<OUT
AOXChain demo flow completed.
profile=${AOXC_BOOTSTRAP_PROFILE}
seeded_account=${AOXC_NEW_ACCOUNT_ID}
transfer_to=${AOXC_TRANSFER_TO}
transfer_amount=${AOXC_TRANSFER_AMOUNT}
OUT
