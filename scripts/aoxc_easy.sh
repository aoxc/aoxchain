#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DAEMON_SCRIPT="${SCRIPT_DIR}/network_env_daemon.sh"
AOXC_DATA_ROOT="${AOXC_DATA_ROOT:-${HOME}/.AOXCData}"

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

usage() {
  cat <<'EOF'
AOXC Easy CLI (for everyone: beginner to advanced)

Usage:
  ./scripts/aoxc_easy.sh help
  ./scripts/aoxc_easy.sh doctor
  ./scripts/aoxc_easy.sh start <mainnet|testnet|devnet>
  ./scripts/aoxc_easy.sh start-dual
  ./scripts/aoxc_easy.sh once <mainnet|testnet|devnet>
  ./scripts/aoxc_easy.sh once-dual
  ./scripts/aoxc_easy.sh stop <mainnet|testnet|devnet>
  ./scripts/aoxc_easy.sh stop-dual
  ./scripts/aoxc_easy.sh status <mainnet|testnet|devnet>
  ./scripts/aoxc_easy.sh status-dual
  ./scripts/aoxc_easy.sh restart <mainnet|testnet|devnet>
  ./scripts/aoxc_easy.sh restart-dual
  ./scripts/aoxc_easy.sh logs <mainnet|testnet|devnet>
  ./scripts/aoxc_easy.sh menu
EOF
}

doctor() {
  echo "[doctor] checking AOXC runtime prerequisites..."

  if [[ ! -x "${DAEMON_SCRIPT}" ]]; then
    echo "[doctor][error] missing executable daemon script: ${DAEMON_SCRIPT}"
    return 1
  fi

  local bin
  bin="$(resolve_bin_path || true)"
  if [[ -z "${bin}" ]]; then
    echo "[doctor][warn] AOXC binary not found. run: make package-bin"
  else
    echo "[doctor][ok] AOXC binary: ${bin}"
  fi

  mkdir -p "${AOXC_DATA_ROOT}/logs/network/mainnet" "${AOXC_DATA_ROOT}/logs/network/testnet" "${AOXC_DATA_ROOT}/logs/network/devnet"
  echo "[doctor][ok] log directories prepared"
  echo "[doctor] done"
}

require_env() {
  local env="${1:-}"
  case "${env}" in
    mainnet|testnet|devnet) ;;
    *)
      echo "[easy][error] invalid environment: ${env:-<empty>} (use mainnet|testnet|devnet)" >&2
      exit 2
      ;;
  esac
}

run_dual() {
  local cmd="${1:?missing-cmd}"
  "${SCRIPT_DIR}/network_stack.sh" "${cmd}"
}

cmd="${1:-help}"
env="${2:-}"

case "${cmd}" in
  help|-h|--help)
    usage
    ;;
  doctor)
    doctor
    ;;
  start|once|stop|status)
    require_env "${env}"
    "${DAEMON_SCRIPT}" "${cmd}" "${env}"
    ;;
  start-dual)
    run_dual start
    ;;
  once-dual)
    run_dual once
    ;;
  stop-dual)
    run_dual stop
    ;;
  status-dual)
    run_dual status
    ;;
  restart)
    require_env "${env}"
    "${DAEMON_SCRIPT}" stop "${env}" || true
    "${DAEMON_SCRIPT}" start "${env}"
    ;;
  restart-dual)
    run_dual restart
    ;;
  logs)
    require_env "${env}"
    tail -n 120 -f "${AOXC_DATA_ROOT}/logs/network/${env}/runtime.log"
    ;;
  menu)
    cat <<'EOF'
AOXC EASY MENU
1) make ops-doctor
2) make ops-start-mainnet
3) make ops-start-testnet
4) make ops-start-devnet
5) make ops-start-dual
6) make ops-status-mainnet
7) make ops-status-testnet
8) make ops-status-devnet
9) make ops-status-dual
10) make ops-stop-mainnet
11) make ops-stop-testnet
12) make ops-stop-devnet
13) make ops-stop-dual
14) make ops-logs-mainnet
15) make ops-logs-testnet
16) make ops-logs-devnet
EOF
    ;;
  *)
    echo "[easy][error] unknown command: ${cmd}" >&2
    usage
    exit 2
    ;;
esac
