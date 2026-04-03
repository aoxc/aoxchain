#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

enter_repo_root
run_make_target package-bin >/dev/null
require_executable "${AOXC_REPO_ROOT}/target/release/aoxc"

AOXC_HOME="${AOXC_RUNTIME_ROOT:-${HOME}/.aoxc/runtime}" "${AOXC_REPO_ROOT}/target/release/aoxc" network-smoke
