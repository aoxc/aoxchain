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

  if [[ -x "${HOME}/.AOXCData/bin/aoxc" ]]; then
    printf "%s" "${HOME}/.AOXCData/bin/aoxc"
    return 0
  fi

  if [[ -x "${ROOT_DIR}/bin/aoxc" ]]; then
    printf "%s" "${ROOT_DIR}/bin/aoxc"
    return 0
  fi

  return 1
}

resolve_default_root_seed() {
  local env="${1:?missing-env}"
  local seed_file="${AOXC_DATA_ROOT}/seeds/${env}.root.seed"
  mkdir -p "$(dirname "${seed_file}")"

  if [[ -s "${seed_file}" ]]; then
    tr -d '\r\n' < "${seed_file}"
    return 0
  fi

  local generated_seed
  generated_seed="$(printf "AOXC::ROOT::%s::%s::%s" "${env}" "$(hostname -s 2>/dev/null || echo unknown-host)" "$(date -u +%s)" | sha256sum | awk '{print $1}')"
  printf "%s\n" "${generated_seed}" > "${seed_file}"
  chmod 600 "${seed_file}" || true
  echo "[network-env][bootstrap] root seed generated for env=${env} path=${seed_file}" | tee -a "${RUNTIME_LOG}"
  printf "%s" "${generated_seed}"
}

derive_account_id() {
  local root_seed="${1:?missing-root-seed}"
  local env="${2:?missing-env}"
  local index="${3:?missing-index}"
  printf "AOXC_%s_%s" "${env^^}" "$(printf "%s::%s::%s" "${root_seed}" "${env}" "${index}" | sha256sum | awk '{print substr($1,1,24)}')"
}

materialize_generated_accounts() {
  local env="${1:?missing-env}"
  local root_seed="${2:?missing-root-seed}"
  local desired_count="${3:-5}"
  local accounts_file="${AOXC_HOME_DIR}/identity/accounts.generated.json"
  local ledger_file="${AOXC_HOME_DIR}/support/generated-accounts.json"
  local i account_id account_seed

  mkdir -p "${AOXC_HOME_DIR}/identity" "${AOXC_HOME_DIR}/support"

  {
    echo "{"
    echo "  \"schema_version\": 1,"
    echo "  \"environment\": \"${env}\","
    echo "  \"root_seed_sha256\": \"$(printf "%s" "${root_seed}" | sha256sum | awk '{print $1}')\","
    echo "  \"accounts\": ["
    for ((i = 1; i <= desired_count; i++)); do
      account_id="$(derive_account_id "${root_seed}" "${env}" "${i}")"
      account_seed="$(printf "%s::%s::%s::seed" "${root_seed}" "${env}" "${i}" | sha256sum | awk '{print $1}')"
      cat <<JSON
    {
      "index": ${i},
      "name": "validator-auto-${i}",
      "account_id": "${account_id}",
      "seed_material_sha256": "${account_seed}",
      "initial_balance": "1000000000"
    }$( [[ "${i}" -lt "${desired_count}" ]] && printf "," )
JSON
    done
    echo "  ]"
    echo "}"
  } > "${accounts_file}"

  cp "${accounts_file}" "${ledger_file}"
  echo "[network-env][bootstrap] generated ${desired_count} deterministic accounts from root seed for env=${env}" | tee -a "${RUNTIME_LOG}"
}

patch_genesis_with_generated_accounts() {
  local genesis_file="${AOXC_HOME_DIR}/identity/genesis.json"
  local accounts_file="${AOXC_HOME_DIR}/identity/accounts.generated.json"
  local env="${1:?missing-env}"

  if [[ ! -f "${genesis_file}" || ! -f "${accounts_file}" ]]; then
    echo "[network-env][bootstrap][warn] genesis/account artifact missing; skip genesis account patch" | tee -a "${RUNTIME_LOG}"
    return 0
  fi

  python3 - "${genesis_file}" "${accounts_file}" "${env}" <<'PY'
import json
import sys
from pathlib import Path

genesis_path = Path(sys.argv[1])
accounts_path = Path(sys.argv[2])
env = sys.argv[3]

genesis = json.loads(genesis_path.read_text(encoding="utf-8"))
accounts_doc = json.loads(accounts_path.read_text(encoding="utf-8"))

generated = []
for row in accounts_doc.get("accounts", []):
    generated.append(
        {
            "account_id": row["account_id"],
            "balance": str(row.get("initial_balance", "1000000000")),
            "role": f"generated-validator-{row.get('index', 0)}",
        }
    )

state = genesis.setdefault("state", {})
state["accounts"] = generated if generated else state.get("accounts", [])
bindings = genesis.setdefault("bindings", {})
bindings["accounts_file"] = "accounts.generated.json"
metadata = genesis.setdefault("metadata", {})
metadata["description"] = f"Canonical AOXC {env} genesis configuration with deterministic generated accounts."

genesis_path.write_text(json.dumps(genesis, indent=2), encoding="utf-8")
PY
  echo "[network-env][bootstrap] genesis patched with generated deterministic accounts for env=${env}" | tee -a "${RUNTIME_LOG}"
}

BIN_PATH="$(resolve_bin_path || true)"
AOXC_DATA_ROOT="${AOXC_DATA_ROOT:-${HOME}/.AOXCData}"
if [[ -z "${BIN_PATH}" ]]; then
  echo "[network-env][error] AOXC binary not found. run: make package-bin" >&2
  exit 4
fi

AOXC_HOME_DIR="${AOXC_HOME_DIR:-${AOXC_DATA_ROOT}/home/${TARGET_ENV}}"
LOG_DIR="${LOG_DIR:-${AOXC_DATA_ROOT}/logs/network/${TARGET_ENV}}"
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
  local root_seed
  mkdir -p "${LOG_DIR}" "${AOXC_HOME_DIR}"
  export AOXC_HOME="${AOXC_HOME_DIR}"

  if [[ -f "${AOXC_HOME_DIR}/.bootstrap_done" ]]; then
    echo "[network-env] bootstrap already completed (${TARGET_ENV})" | tee -a "${RUNTIME_LOG}"
    return 0
  fi

  echo "[network-env] bootstrap start env=${TARGET_ENV}" | tee -a "${RUNTIME_LOG}"
  root_seed="$(resolve_default_root_seed "${TARGET_ENV}")"
  materialize_generated_accounts "${TARGET_ENV}" "${root_seed}" "${GENESIS_ACCOUNT_COUNT:-5}"

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
  patch_genesis_with_generated_accounts "${TARGET_ENV}"

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
