#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

usage() {
  cat <<'OUT'
Usage: scripts/chain_create.sh [--dry-run] [--verbose] [--print-env]
Initializes chain state via `make chain-init`.
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

PROFILE="${AOXC_BOOTSTRAP_PROFILE:-localnet}"
VALIDATOR_NAME="${AOXC_VALIDATOR_NAME:-validator-01}"
VALIDATOR_PASSWORD="${AOXC_VALIDATOR_PASSWORD:-ChangeMe#2026}"

require_non_empty "${PROFILE}" "AOXC_BOOTSTRAP_PROFILE"
require_non_empty "${VALIDATOR_NAME}" "AOXC_VALIDATOR_NAME"
require_non_empty "${VALIDATOR_PASSWORD}" "AOXC_VALIDATOR_PASSWORD"

if [[ "${PRINT_ENV}" == "1" ]]; then
  printf 'AOXC_BOOTSTRAP_PROFILE=%s\nAOXC_VALIDATOR_NAME=%s\n' "${PROFILE}" "${VALIDATOR_NAME}"
fi

run_make_target chain-init \
  AOXC_BOOTSTRAP_PROFILE="${PROFILE}" \
  AOXC_VALIDATOR_NAME="${VALIDATOR_NAME}" \
  AOXC_VALIDATOR_PASSWORD="${VALIDATOR_PASSWORD}"
