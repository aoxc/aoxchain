#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

usage() {
  cat <<'OUT'
Usage: scripts/wallet_seed.sh [--dry-run] [--verbose] [--print-env]
Adds a funded account to genesis via `make chain-add-account`.
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

ACCOUNT_ID="${AOXC_NEW_ACCOUNT_ID:-AOXC_USER_ALICE}"
BALANCE="${AOXC_NEW_ACCOUNT_BALANCE:-1000000}"
ROLE="${AOXC_NEW_ACCOUNT_ROLE:-user}"

require_non_empty "${ACCOUNT_ID}" "AOXC_NEW_ACCOUNT_ID"
require_positive_uint "${BALANCE}" "AOXC_NEW_ACCOUNT_BALANCE"
require_non_empty "${ROLE}" "AOXC_NEW_ACCOUNT_ROLE"

if [[ "${PRINT_ENV}" == "1" ]]; then
  printf 'AOXC_NEW_ACCOUNT_ID=%s\nAOXC_NEW_ACCOUNT_BALANCE=%s\nAOXC_NEW_ACCOUNT_ROLE=%s\n' "${ACCOUNT_ID}" "${BALANCE}" "${ROLE}"
fi

run_make_target chain-add-account \
  AOXC_NEW_ACCOUNT_ID="${ACCOUNT_ID}" \
  AOXC_NEW_ACCOUNT_BALANCE="${BALANCE}" \
  AOXC_NEW_ACCOUNT_ROLE="${ROLE}"
