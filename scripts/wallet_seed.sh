#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

ACCOUNT_ID="${AOXC_NEW_ACCOUNT_ID:-AOXC_USER_ALICE}"
BALANCE="${AOXC_NEW_ACCOUNT_BALANCE:-1000000}"
ROLE="${AOXC_NEW_ACCOUNT_ROLE:-user}"

make --no-print-directory chain-add-account \
  AOXC_NEW_ACCOUNT_ID="${ACCOUNT_ID}" \
  AOXC_NEW_ACCOUNT_BALANCE="${BALANCE}" \
  AOXC_NEW_ACCOUNT_ROLE="${ROLE}"
