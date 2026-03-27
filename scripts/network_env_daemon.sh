#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

COMMAND="${1:-}"
TARGET_ENV="${2:-devnet}"

if [[ -z "${COMMAND}" ]]; then
  echo "usage: $0 <start|once|status|stop> <mainnet|testnet|devnet>" >&2
  exit 2
fi

case "${TARGET_ENV}" in
  mainnet|testnet|devnet) ;;
  *)
    echo "[network-env] invalid environment: ${TARGET_ENV}" >&2
    exit 3
    ;;
esac

resolve_bin_path() {
  if [[ -n "${BIN_PATH:-}" && -x "${BIN_PATH}" ]]; then
    printf "%s" "${BIN_PATH}"
    return 0
  fi

  if [[ -x "${HOME}/.aoxc/bin/aoxc" ]]; then
    printf "%s" "${HOME}/.aoxc/bin/aoxc"
    return 0
  fi

  if [[ -x "${ROOT_DIR}/bin/aoxc" ]]; then
    printf "%s" "${ROOT_DIR}/bin/aoxc"
    return 0
  fi

  return 1
}

BIN_PATH="$(resolve_bin_path || true)"
if [[ -z "${BIN_PATH}" ]]; then
  echo "[network-env][error] AOXC binary not found. run: make package-bin" >&2
  exit 4
fi

AOXC_HOME_DIR="${AOXC_HOME_DIR:-${ROOT_DIR}/.aoxc-${TARGET_ENV}}"
LOG_DIR="${LOG_DIR:-${ROOT_DIR}/logs/network/${TARGET_ENV}}"
PID_FILE="${LOG_DIR}/daemon.pid"
RUNTIME_LOG="${LOG_DIR}/runtime.log"

case "${TARGET_ENV}" in
  mainnet)
    KEY_PROFILE="mainnet"
    VALIDATOR_NAME="validator-mainnet-01"
    CHAIN_NUM="1"
    BLOCK_TIME="3"
    TREASURY="1000000000000000"
    ;;
  testnet)
    KEY_PROFILE="testnet"
    VALIDATOR_NAME="validator-testnet-01"
    CHAIN_NUM="1001"
    BLOCK_TIME="4"
    TREASURY="10000000000000"
    ;;
  devnet)
    KEY_PROFILE="testnet"
    VALIDATOR_NAME="validator-devnet-01"
    CHAIN_NUM="2001"
    BLOCK_TIME="2"
    TREASURY="1000000000000"
    ;;
esac

bootstrap_env() {
  mkdir -p "${LOG_DIR}" "${AOXC_HOME_DIR}"
  export AOXC_HOME="${AOXC_HOME_DIR}"

  if [[ -f "${AOXC_HOME_DIR}/.bootstrap_done" ]]; then
    echo "[network-env] bootstrap already completed (${TARGET_ENV})" | tee -a "${RUNTIME_LOG}"
    return 0
  fi

  echo "[network-env] bootstrap start env=${TARGET_ENV}" | tee -a "${RUNTIME_LOG}"
  "${BIN_PATH}" key-bootstrap \
    --profile "${KEY_PROFILE}" \
    --name "${VALIDATOR_NAME}" \
    --password "TEST#Secure2026!" \
    --home "${AOXC_HOME_DIR}" 2>&1 | tee -a "${RUNTIME_LOG}"

  "${BIN_PATH}" genesis-init \
    --home "${AOXC_HOME_DIR}" \
    --chain-num "${CHAIN_NUM}" \
    --block-time "${BLOCK_TIME}" \
    --treasury "${TREASURY}" \
    --native-symbol AOXC \
    --native-decimals 18 \
    --settlement-network xlayer \
    --xlayer-token 0xeb9580c3946bb47d73aae1d4f7a94148b554b2f4 \
    --xlayer-main-contract 0x97bdd1fd1caf756e00efd42eba9406821465b365 \
    --xlayer-multisig 0x20c0dd8b6559912acfac2ce061b8d5b19db8ca84 \
    --equivalence-mode 1:1 2>&1 | tee -a "${RUNTIME_LOG}"

  "${BIN_PATH}" node-bootstrap --home "${AOXC_HOME_DIR}" 2>&1 | tee -a "${RUNTIME_LOG}"

  touch "${AOXC_HOME_DIR}/.bootstrap_done"
  echo "[network-env] bootstrap done env=${TARGET_ENV}" | tee -a "${RUNTIME_LOG}"
}

run_once() {
  export AOXC_HOME="${AOXC_HOME_DIR}"
  tx_payload="AOXC_${TARGET_ENV}_$(date +%s)"
  "${BIN_PATH}" produce-once --tx "${tx_payload}" 2>&1 | tee -a "${RUNTIME_LOG}"
  "${BIN_PATH}" network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0 --payload "HEALTH_${TARGET_ENV}" 2>&1 | tee -a "${RUNTIME_LOG}"
}

start_daemon() {
  if [[ -f "${PID_FILE}" ]] && kill -0 "$(cat "${PID_FILE}")" >/dev/null 2>&1; then
    echo "[network-env] already running env=${TARGET_ENV} pid=$(cat "${PID_FILE}")"
    return 0
  fi

  bootstrap_env

  (
    export AOXC_HOME="${AOXC_HOME_DIR}"
    while true; do
      tx_payload="AOXC_${TARGET_ENV}_DAEMON_$(date +%s)"
      "${BIN_PATH}" produce-once --tx "${tx_payload}" >>"${RUNTIME_LOG}" 2>&1 || true
      "${BIN_PATH}" network-smoke --timeout-ms 3000 --bind-host 127.0.0.1 --port 0 --payload "HEALTH_${TARGET_ENV}" >>"${RUNTIME_LOG}" 2>&1 || true
      sleep 2
    done
  ) &

  echo $! > "${PID_FILE}"
  echo "[network-env] started env=${TARGET_ENV} pid=$(cat "${PID_FILE}") log=${RUNTIME_LOG}"
}

status_daemon() {
  if [[ -f "${PID_FILE}" ]] && kill -0 "$(cat "${PID_FILE}")" >/dev/null 2>&1; then
    echo "[network-env] running env=${TARGET_ENV} pid=$(cat "${PID_FILE}")"
  else
    echo "[network-env] stopped env=${TARGET_ENV}"
  fi
}

stop_daemon() {
  if [[ -f "${PID_FILE}" ]]; then
    pid="$(cat "${PID_FILE}")"
    if kill -0 "${pid}" >/dev/null 2>&1; then
      kill "${pid}" >/dev/null 2>&1 || true
      sleep 1
      kill -9 "${pid}" >/dev/null 2>&1 || true
      echo "[network-env] stopped env=${TARGET_ENV} pid=${pid}"
    fi
    rm -f "${PID_FILE}"
  else
    echo "[network-env] no pid file for env=${TARGET_ENV}"
  fi
}

case "${COMMAND}" in
  start)
    start_daemon
    ;;
  once)
    bootstrap_env
    run_once
    ;;
  status)
    status_daemon
    ;;
  stop)
    stop_daemon
    ;;
  *)
    echo "[network-env] unknown command: ${COMMAND}" >&2
    exit 5
    ;;
esac
