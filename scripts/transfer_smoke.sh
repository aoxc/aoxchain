#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

TO="${AOXC_TRANSFER_TO:-bob}"
AMOUNT="${AOXC_TRANSFER_AMOUNT:-100}"

make --no-print-directory package-bin >/dev/null
AOXC_HOME="${AOXC_RUNTIME_ROOT:-${HOME}/.aoxc/runtime}" "${ROOT_DIR}/target/release/aoxc" treasury-transfer --to "${TO}" --amount "${AMOUNT}"
