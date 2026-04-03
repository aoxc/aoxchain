#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

TO="${AOXC_TRANSFER_TO:-bob}"
AMOUNT="${AOXC_TRANSFER_AMOUNT:-100}"

require_non_empty "${TO}" "AOXC_TRANSFER_TO"
require_uint "${AMOUNT}" "AOXC_TRANSFER_AMOUNT"

enter_repo_root
run_make_target package-bin >/dev/null
require_executable "${AOXC_REPO_ROOT}/target/release/aoxc"

AOXC_HOME="${AOXC_RUNTIME_ROOT:-${HOME}/.aoxc/runtime}" "${AOXC_REPO_ROOT}/target/release/aoxc" treasury-transfer --to "${TO}" --amount "${AMOUNT}"
