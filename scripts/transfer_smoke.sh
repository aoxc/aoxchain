#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

usage() {
  cat <<'OUT'
Usage: scripts/transfer_smoke.sh [--dry-run] [--verbose] [--print-env]
Builds runtime binary and runs `aoxc treasury-transfer`.
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

TO="${AOXC_TRANSFER_TO:-bob}"
AMOUNT="${AOXC_TRANSFER_AMOUNT:-100}"
RUNTIME_HOME="${AOXC_RUNTIME_ROOT:-${HOME}/.aoxc/runtime}"

require_non_empty "${TO}" "AOXC_TRANSFER_TO"
require_positive_uint "${AMOUNT}" "AOXC_TRANSFER_AMOUNT"

if [[ "${PRINT_ENV}" == "1" ]]; then
  printf 'AOXC_TRANSFER_TO=%s\nAOXC_TRANSFER_AMOUNT=%s\nAOXC_HOME=%s\n' "${TO}" "${AMOUNT}" "${RUNTIME_HOME}"
fi

if [[ "${AOXC_SCRIPT_DRY_RUN}" == "1" ]]; then
  run_make_target package-bin >/dev/null
  run_cmd env AOXC_HOME="${RUNTIME_HOME}" "${AOXC_REPO_ROOT}/target/release/aoxc" treasury-transfer --to "${TO}" --amount "${AMOUNT}"
  exit 0
fi

run_make_target package-bin >/dev/null
require_executable "${AOXC_REPO_ROOT}/target/release/aoxc"
run_cmd env AOXC_HOME="${RUNTIME_HOME}" "${AOXC_REPO_ROOT}/target/release/aoxc" treasury-transfer --to "${TO}" --amount "${AMOUNT}"
