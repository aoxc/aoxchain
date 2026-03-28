#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

resolve_bin_path() {
  if [[ -n "${BIN_PATH:-}" && -x "${BIN_PATH}" ]]; then
    printf "%s" "${BIN_PATH}"
    return 0
  fi

  if [[ -x "${HOME}/.AOXCData/bin/aoxc" ]]; then
    printf "%s" "${HOME}/.AOXCData/bin/aoxc"
    return 0
  fi

  if [[ -x "./bin/aoxc" ]]; then
    printf "%s" "./bin/aoxc"
    return 0
  fi

  return 1
}

BIN_PATH="$(resolve_bin_path || true)"
TX_PREFIX="${TX_PREFIX:-auto-tx}"
SLEEP_SECS="${SLEEP_SECS:-2}"
MAX_ROUNDS="${MAX_ROUNDS:-0}"
LOG_FILE="${LOG_FILE:-./logs/continuous-producer.log}"

if [[ -z "${BIN_PATH}" || ! -x "${BIN_PATH}" ]]; then
  echo "[error] binary is not executable: ${BIN_PATH}" >&2
  echo "Build/install it with: make package-bin" >&2
  exit 2
fi

mkdir -p "$(dirname "${LOG_FILE}")"

echo "[producer] start: bin=${BIN_PATH} sleep=${SLEEP_SECS}s max_rounds=${MAX_ROUNDS}" | tee -a "${LOG_FILE}"

round=0
while true; do
  round=$((round + 1))
  tx_payload="${TX_PREFIX}-${round}-$(date +%s)"

  echo "[producer][round=${round}] produce-once tx=${tx_payload}" | tee -a "${LOG_FILE}"
  set +e
  "${BIN_PATH}" produce-once --tx "${tx_payload}" 2>&1 | tee -a "${LOG_FILE}"
  exit_code=${PIPESTATUS[0]}
  set -e

  if [[ ${exit_code} -ne 0 ]]; then
    echo "[producer][round=${round}] ERROR code=${exit_code}" | tee -a "${LOG_FILE}"
  else
    echo "[producer][round=${round}] OK" | tee -a "${LOG_FILE}"
  fi

  if [[ "${MAX_ROUNDS}" -gt 0 && "${round}" -ge "${MAX_ROUNDS}" ]]; then
    echo "[producer] max rounds reached: ${MAX_ROUNDS}" | tee -a "${LOG_FILE}"
    break
  fi

  sleep "${SLEEP_SECS}"
done
