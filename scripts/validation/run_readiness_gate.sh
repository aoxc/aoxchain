#!/usr/bin/env bash
set -euo pipefail

AOXC_HOME="${AOXC_HOME:-$(pwd)/.artifacts/readiness-home}"
AOXC_BIN="${AOXC_BIN:-cargo run -q -p aoxcmd --}"
PASSWORD="${PASSWORD:-Readiness#2026!}"
PROFILE="${PROFILE:-mainnet}"
VALIDATOR_NAME="${VALIDATOR_NAME:-validator-readiness}"

mkdir -p "${AOXC_HOME}"
export AOXC_HOME

${AOXC_BIN} production-bootstrap \
  --password "${PASSWORD}" \
  --profile "${PROFILE}" \
  --name "${VALIDATOR_NAME}" \
  --bind-host 0.0.0.0

${AOXC_BIN} mainnet-readiness --enforce --format json
${AOXC_BIN} full-surface-readiness --enforce --format json
