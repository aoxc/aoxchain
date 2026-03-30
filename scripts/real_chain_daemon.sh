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
AOXC_DATA_ROOT="${AOXC_DATA_ROOT:-${HOME}/.AOXCData}"
AOXC_HOME_DIR="${AOXC_HOME_DIR:-${AOXC_DATA_ROOT}/home/real}"
LOG_DIR="${LOG_DIR:-${AOXC_DATA_ROOT}/logs/real-chain}"
MAX_CYCLES="${MAX_CYCLES:-0}"   # 0 means infinite
ROUND_PER_CYCLE="${ROUND_PER_CYCLE:-60}"
SLEEP_MS="${SLEEP_MS:-1000}"
NETWORK_ROUNDS="${NETWORK_ROUNDS:-5}"
NETWORK_TIMEOUT_MS="${NETWORK_TIMEOUT_MS:-3000}"
NETWORK_PAUSE_MS="${NETWORK_PAUSE_MS:-250}"

if [[ -z "${BIN_PATH}" || ! -x "${BIN_PATH}" ]]; then
  echo "[real-chain][error] binary is not executable: ${BIN_PATH}" >&2
  echo "[real-chain][hint] run: make package-bin (installs \$HOME/.AOXCData/bin/aoxc)" >&2
  exit 2
fi

mkdir -p "${LOG_DIR}"
RUNTIME_LOG="${LOG_DIR}/runtime.log"
HEALTH_LOG="${LOG_DIR}/health.log"

export AOXC_HOME="${AOXC_HOME_DIR}"

echo "[real-chain] ===== startup =====" | tee -a "${RUNTIME_LOG}"
echo "[real-chain] bin=${BIN_PATH}" | tee -a "${RUNTIME_LOG}"
echo "[real-chain] AOXC_HOME=${AOXC_HOME}" | tee -a "${RUNTIME_LOG}"
echo "[real-chain] MAX_CYCLES=${MAX_CYCLES} ROUND_PER_CYCLE=${ROUND_PER_CYCLE} SLEEP_MS=${SLEEP_MS}" | tee -a "${RUNTIME_LOG}"

echo "[real-chain] key-bootstrap (testnet profile)" | tee -a "${RUNTIME_LOG}"
"${BIN_PATH}" key-bootstrap \
  --profile testnet \
  --name validator-real-01 \
  --password "TEST#Secure2026!" \
  --home "${AOXC_HOME}" 2>&1 | tee -a "${RUNTIME_LOG}"

echo "[real-chain] genesis-init" | tee -a "${RUNTIME_LOG}"
"${BIN_PATH}" genesis-init \
  --home "${AOXC_HOME}" \
  --chain-num 1001 \
  --block-time 6 \
  --treasury 1000000000000 \
  --native-symbol AOXC \
  --native-decimals 18 \
  --settlement-network xlayer \
  --xlayer-token 0xeb9580c3946bb47d73aae1d4f7a94148b554b2f4 \
  --xlayer-main-contract 0x97bdd1fd1caf756e00efd42eba9406821465b365 \
  --xlayer-multisig 0x20c0dd8b6559912acfac2ce061b8d5b19db8ca84 \
  --equivalence-mode 1:1 2>&1 | tee -a "${RUNTIME_LOG}"

echo "[real-chain] node-bootstrap" | tee -a "${RUNTIME_LOG}"
"${BIN_PATH}" node-bootstrap --home "${AOXC_HOME}" 2>&1 | tee -a "${RUNTIME_LOG}"

cycle=0
while true; do
  cycle=$((cycle + 1))
  ts="$(date -Iseconds)"

  echo "[real-chain][${ts}] cycle=${cycle} produce loop start" | tee -a "${RUNTIME_LOG}"
  node_exit=0
  for local_round in $(seq 1 "${ROUND_PER_CYCLE}"); do
    tx_payload="AOXC_REAL_${cycle}_${local_round}_$(date +%s)"
    set +e
    "${BIN_PATH}" produce-once --tx "${tx_payload}" 2>&1 | tee -a "${RUNTIME_LOG}"
    cmd_exit=${PIPESTATUS[0]}
    set -e

    if [[ ${cmd_exit} -ne 0 ]]; then
      node_exit=${cmd_exit}
      echo "[real-chain][warn] cycle=${cycle} produce-once failed round=${local_round} code=${cmd_exit}" | tee -a "${RUNTIME_LOG}"
      break
    fi

    if [[ ${local_round} -lt ${ROUND_PER_CYCLE} ]]; then
      sleep "$(awk "BEGIN { printf \"%.3f\", ${SLEEP_MS}/1000 }")"
    fi
  done

  echo "[real-chain][${ts}] cycle=${cycle} network/health probe start" | tee -a "${HEALTH_LOG}"
  network_exit=0
  for health_round in $(seq 1 "${NETWORK_ROUNDS}"); do
    set +e
    "${BIN_PATH}" network-smoke \
      --timeout-ms "${NETWORK_TIMEOUT_MS}" \
      --bind-host 127.0.0.1 \
      --port 0 \
      --payload "AOXC_HEALTH_CYCLE_${cycle}_${health_round}" 2>&1 | tee -a "${HEALTH_LOG}"
    cmd_exit=${PIPESTATUS[0]}
    set -e

    if [[ ${cmd_exit} -ne 0 ]]; then
      network_exit=${cmd_exit}
      break
    fi

    sleep "$(awk "BEGIN { printf \"%.3f\", ${NETWORK_PAUSE_MS}/1000 }")"
  done

  if [[ ${node_exit} -ne 0 || ${network_exit} -ne 0 ]]; then
    echo "[real-chain][warn] cycle=${cycle} node_exit=${node_exit} network_exit=${network_exit}" | tee -a "${RUNTIME_LOG}" "${HEALTH_LOG}"
  else
    echo "[real-chain][ok] cycle=${cycle} node+network checks passed" | tee -a "${RUNTIME_LOG}" "${HEALTH_LOG}"
  fi

  if [[ "${MAX_CYCLES}" -gt 0 && "${cycle}" -ge "${MAX_CYCLES}" ]]; then
    echo "[real-chain] max cycles reached=${MAX_CYCLES}, exiting" | tee -a "${RUNTIME_LOG}"
    break
  fi
done
