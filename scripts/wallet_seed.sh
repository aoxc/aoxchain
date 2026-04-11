#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

ACCOUNT_ID="${AOXC_NEW_ACCOUNT_ID:-AOXC_USER_ALICE}"
BALANCE="${AOXC_NEW_ACCOUNT_BALANCE:-1000000}"
ROLE="${AOXC_NEW_ACCOUNT_ROLE:-user}"
ALLOW_ZERO_BALANCE="${AOXC_ALLOW_ZERO_BALANCE:-0}"
ALLOWED_ROLES=(
  treasury
  validator
  system
  user
  governance
  forge
  quorum
  seal
  archive
  sentinel
  relay
  pocket
)

require_non_empty "${ACCOUNT_ID}" "AOXC_NEW_ACCOUNT_ID"
require_uint "${BALANCE}" "AOXC_NEW_ACCOUNT_BALANCE"
require_non_empty "${ROLE}" "AOXC_NEW_ACCOUNT_ROLE"

if [[ ! "${ACCOUNT_ID}" =~ ^[A-Za-z0-9_.:-]{3,64}$ ]]; then
  die "AOXC_NEW_ACCOUNT_ID must match ^[A-Za-z0-9_.:-]{3,64}$ (got '${ACCOUNT_ID}')" 2
fi

if (( BALANCE == 0 )); then
  if [[ "${ALLOW_ZERO_BALANCE}" != "1" ]]; then
    die "AOXC_NEW_ACCOUNT_BALANCE must be greater than zero (set AOXC_ALLOW_ZERO_BALANCE=1 to override)" 2
  fi
fi

ROLE="$(printf '%s' "${ROLE}" | tr '[:upper:]' '[:lower:]')"
if [[ ! " ${ALLOWED_ROLES[*]} " =~ [[:space:]]${ROLE}[[:space:]] ]]; then
  die "AOXC_NEW_ACCOUNT_ROLE '${ROLE}' is unsupported. Allowed roles: ${ALLOWED_ROLES[*]}" 2
fi

run_make_target chain-add-account \
  AOXC_NEW_ACCOUNT_ID="${ACCOUNT_ID}" \
  AOXC_NEW_ACCOUNT_BALANCE="${BALANCE}" \
  AOXC_NEW_ACCOUNT_ROLE="${ROLE}"
