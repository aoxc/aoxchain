#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

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

require_non_empty "${TO}" "AOXC_TRANSFER_TO"
require_uint "${AMOUNT}" "AOXC_TRANSFER_AMOUNT"

enter_repo_root
run_make_target package-bin >/dev/null
require_executable "${AOXC_REPO_ROOT}/target/release/aoxc"

AOXC_HOME="${AOXC_RUNTIME_ROOT:-${HOME}/.aoxc/runtime}" "${AOXC_REPO_ROOT}/target/release/aoxc" treasury-transfer --to "${TO}" --amount "${AMOUNT}"
