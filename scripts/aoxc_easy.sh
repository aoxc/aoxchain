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
  ./scripts/aoxc_easy.sh start [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh start-dual
  ./scripts/aoxc_easy.sh once [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh once-dual
  ./scripts/aoxc_easy.sh stop [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh stop-dual
  ./scripts/aoxc_easy.sh status [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh status-dual
  ./scripts/aoxc_easy.sh restart [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh restart-dual
  ./scripts/aoxc_easy.sh logs [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh auto-start
  ./scripts/aoxc_easy.sh auto-once
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

resolve_env_or_default() {
  local candidate="${1:-${AOXC_ENV:-devnet}}"
  case "${candidate}" in
    mainnet|testnet|devnet)
      printf "%s" "${candidate}"
      ;;
    *)
      echo "[easy][warn] unknown AOXC_ENV=${candidate}; falling back to devnet"
      printf "devnet"
      ;;
  esac
}

run_dual() {
  local cmd="${1:?missing-cmd}"
  "${SCRIPT_DIR}/network_stack.sh" "${cmd}"
}

print_dashboard_header() {
  echo
  echo "=============================================================="
  echo "                   AOXC OPERATIONS DASHBOARD                 "
  echo "=============================================================="
  echo "Data Root : ${AOXC_DATA_ROOT}"
  echo "Binary    : $(resolve_bin_path || echo "<not-found>")"
  echo "AOXC_ENV  : ${AOXC_ENV:-devnet}"
  echo "Time (UTC): $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "--------------------------------------------------------------"
  printf "%-10s | %-8s | %-10s | %-10s | %-10s\n" "ENV" "DAEMON" "GENESIS" "ACCOUNTS" "LAST-LOG"
  echo "--------------------------------------------------------------"
}

env_row() {
  local env="${1:?missing-env}"
  local log_path="${AOXC_DATA_ROOT}/logs/network/${env}/runtime.log"
  local home_path="${AOXC_DATA_ROOT}/home/${env}"
  local daemon_status genesis_status accounts_status last_log

  daemon_status="$("${DAEMON_SCRIPT}" status "${env}" 2>/dev/null | awk '{print $2}' || true)"
  [[ "${daemon_status}" == "running" ]] || daemon_status="stopped"
  [[ -f "${home_path}/identity/genesis.json" ]] && genesis_status="present" || genesis_status="missing"
  [[ -f "${home_path}/identity/accounts.generated.json" ]] && accounts_status="present" || accounts_status="missing"
  if [[ -f "${log_path}" ]]; then
    last_log="$(tail -n 1 "${log_path}" 2>/dev/null | cut -c1-40)"
    [[ -n "${last_log}" ]] || last_log="<empty>"
  else
    last_log="<none>"
  fi

  printf "%-10s | %-8s | %-10s | %-10s | %-10s\n" "${env}" "${daemon_status}" "${genesis_status}" "${accounts_status}" "${last_log}"
}

dashboard() {
  print_dashboard_header
  env_row mainnet
  env_row testnet
  env_row devnet
  echo "--------------------------------------------------------------"
  echo "Quick actions:"
  echo "  - Full auto start : ./scripts/aoxc_easy.sh flow devnet"
  echo "  - Env auto start  : ./scripts/aoxc_easy.sh auto-start"
  echo "  - Make control    : ./scripts/aoxc_easy.sh makectl start testnet"
  echo "=============================================================="
}

ensure_binary_or_build() {
  local bin_path
  bin_path="$(resolve_bin_path || true)"
  if [[ -n "${bin_path}" ]]; then
    return 0
  fi

  echo "[easy][auto] AOXC binary missing. running: make package-bin"
  (cd "${ROOT_DIR}" && make package-bin)
}

full_auto_flow() {
  local env
  env="$(resolve_env_or_default "${1:-}")"
  echo "[easy][flow] starting full auto flow env=${env}"
  doctor || true
  ensure_binary_or_build
  "${DAEMON_SCRIPT}" start "${env}"
  "${DAEMON_SCRIPT}" status "${env}"
  echo "[easy][flow] completed env=${env}. Dashboard:"
  dashboard
}

makectl() {
  local action="${1:-}"
  local env
  env="$(resolve_env_or_default "${2:-}")"
  local target=""

  case "${action}" in
    start|once|stop|status|restart)
      target="ops-${action}-${env}"
      ;;
    *)
      echo "[easy][error] invalid makectl action: ${action} (use start|once|stop|status|restart)"
      return 2
      ;;
  esac

  echo "[easy][makectl] executing make ${target}"
  (cd "${ROOT_DIR}" && make "${target}")
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
    env="$(resolve_env_or_default "${env}")"
    echo "[easy] command=${cmd} env=${env}"
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
    env="$(resolve_env_or_default "${env}")"
    echo "[easy] command=restart env=${env}"
    "${DAEMON_SCRIPT}" stop "${env}" || true
    "${DAEMON_SCRIPT}" start "${env}"
    ;;
  restart-dual)
    run_dual restart
    ;;
  logs)
    env="$(resolve_env_or_default "${env}")"
    echo "[easy] following runtime logs env=${env}"
    tail -n 120 -f "${AOXC_DATA_ROOT}/logs/network/${env}/runtime.log"
    ;;
  auto-start)
    env="$(resolve_env_or_default "${AOXC_ENV:-}")"
    echo "[easy] auto-start selected env=${env}"
    "${DAEMON_SCRIPT}" start "${env}"
    ;;
  auto-once)
    env="$(resolve_env_or_default "${AOXC_ENV:-}")"
    echo "[easy] auto-once selected env=${env}"
    "${DAEMON_SCRIPT}" once "${env}"
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
17) make ops-dashboard
18) make ops-flow-devnet
19) make ops-flow-testnet
20) make ops-flow-mainnet
EOF
    ;;
  *)
    echo "[easy][error] unknown command: ${cmd}" >&2
    usage
    exit 2
    ;;
esac
