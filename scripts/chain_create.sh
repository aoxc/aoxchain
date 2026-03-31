#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

PROFILE="${AOXC_BOOTSTRAP_PROFILE:-localnet}"
VALIDATOR_NAME="${AOXC_VALIDATOR_NAME:-validator-01}"
VALIDATOR_PASSWORD="${AOXC_VALIDATOR_PASSWORD:-ChangeMe#2026}"

make --no-print-directory chain-init \
  AOXC_BOOTSTRAP_PROFILE="${PROFILE}" \
  AOXC_VALIDATOR_NAME="${VALIDATOR_NAME}" \
  AOXC_VALIDATOR_PASSWORD="${VALIDATOR_PASSWORD}"
