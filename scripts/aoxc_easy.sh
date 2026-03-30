#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Provide a simplified operator-facing AOXC control surface for common
#   lifecycle, dashboard, and convenience operations.
#
# Scope:
#   - Expose easy commands for single-environment and dual-environment control
#   - Provide prerequisite diagnostics for operator workflows
#   - Provide a lightweight operational dashboard
#   - Provide convenience wrappers around Makefile operator targets
#
# Exit Codes:
#   0  Successful completion
#   1  General operational failure
#   2  Invalid invocation
#   3  Invalid environment selection
#   4  Required daemon script is missing or not executable
#   5  AOXC binary resolution failure during enforced checks
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly DAEMON_SCRIPT="${SCRIPT_DIR}/network_env_daemon.sh"
readonly STACK_SCRIPT="${SCRIPT_DIR}/network_stack.sh"
readonly SUPPORTED_ENVS=("mainnet" "testnet" "devnet")

AOXC_DATA_ROOT="${AOXC_DATA_ROOT:-${HOME}/.AOXCData}"

log_info() {
  printf '[easy][info] %s\n' "$*"
}

log_warn() {
  printf '[easy][warn] %s\n' "$*" >&2
}

log_error() {
  printf '[easy][error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"

  log_error "${message}"
  exit "${exit_code}"
}

print_usage() {
  cat <<'EOF'
AOXC Easy CLI

Usage:
  ./scripts/aoxc_easy.sh help
  ./scripts/aoxc_easy.sh doctor
  ./scripts/aoxc_easy.sh start [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh once [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh stop [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh status [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh restart [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh logs [mainnet|testnet|devnet]

  ./scripts/aoxc_easy.sh start-dual
  ./scripts/aoxc_easy.sh once-dual
  ./scripts/aoxc_easy.sh stop-dual
  ./scripts/aoxc_easy.sh status-dual
  ./scripts/aoxc_easy.sh restart-dual

  ./scripts/aoxc_easy.sh auto-start
  ./scripts/aoxc_easy.sh auto-once
  ./scripts/aoxc_easy.sh dashboard
  ./scripts/aoxc_easy.sh flow [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh makectl <start|once|stop|status|restart> [mainnet|testnet|devnet]
  ./scripts/aoxc_easy.sh menu
EOF
}

require_executable_script() {
  local script_path="$1"

  [[ -f "${script_path}" ]] || die "Required script does not exist: ${script_path}" 4
  [[ -x "${script_path}" ]] || die "Required script is not executable: ${script_path}" 4
}

resolve_bin_path() {
  # The binary resolution order is intentionally explicit to preserve
  # predictable operator behavior across packaged and repository-local runtimes.
  if [[ -n "${BIN_PATH:-}" ]]; then
    [[ -x "${BIN_PATH}" ]] || die "BIN_PATH is set but not executable: ${BIN_PATH}" 5
    printf '%s\n' "${BIN_PATH}"
    return 0
  fi

  if [[ -x "${HOME}/.AOXCData/bin/aoxc" ]]; then
    printf '%s\n' "${HOME}/.AOXCData/bin/aoxc"
    return 0
  fi

  if [[ -x "${ROOT_DIR}/bin/aoxc" ]]; then
    printf '%s\n' "${ROOT_DIR}/bin/aoxc"
    return 0
  fi

  return 1
}

require_environment() {
  local env_name="${1:-}"

  case "${env_name}" in
    mainnet|testnet|devnet)
      ;;
    *)
      die "Invalid environment: ${env_name:-<empty>}. Use mainnet, testnet, or devnet." 3
      ;;
  esac
}

resolve_env_or_default() {
  local candidate="${1:-${AOXC_ENV:-devnet}}"

  case "${candidate}" in
    mainnet|testnet|devnet)
      printf '%s\n' "${candidate}"
      ;;
    *)
      log_warn "Unknown AOXC_ENV='${candidate}'. Falling back to 'devnet'."
      printf 'devnet\n'
      ;;
  esac
}

prepare_log_directories() {
  local env_name=''

  for env_name in "${SUPPORTED_ENVS[@]}"; do
    mkdir -p "${AOXC_DATA_ROOT}/logs/network/${env_name}"
  done
}

run_doctor() {
  local bin_path=''

  log_info "Checking AOXC runtime prerequisites."

  require_executable_script "${DAEMON_SCRIPT}"
  require_executable_script "${STACK_SCRIPT}"

  if bin_path="$(resolve_bin_path 2>/dev/null)"; then
    printf '[doctor][ok] AOXC binary: %s\n' "${bin_path}"
  else
    printf '[doctor][warn] AOXC binary not found. Run: make package-bin\n'
  fi

  prepare_log_directories
  printf '[doctor][ok] network log directories prepared under %s\n' "${AOXC_DATA_ROOT}/logs/network"
  printf '[doctor][ok] prerequisite inspection completed\n'
}

run_dual() {
  local command="$1"

  require_executable_script "${STACK_SCRIPT}"
  "${STACK_SCRIPT}" "${command}"
}

print_dashboard_header() {
  printf '\n'
  printf '==============================================================\n'
  printf '                   AOXC OPERATIONS DASHBOARD                  \n'
  printf '==============================================================\n'
  printf 'Data Root : %s\n' "${AOXC_DATA_ROOT}"
  printf 'Binary    : %s\n' "$(resolve_bin_path 2>/dev/null || printf '<not-found>')"
  printf 'AOXC_ENV  : %s\n' "${AOXC_ENV:-devnet}"
  printf 'Time (UTC): %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf '%s\n' '--------------------------------------------------------------'
  printf '%-10s | %-8s | %-10s | %-10s | %-20s\n' "ENV" "DAEMON" "GENESIS" "ACCOUNTS" "LAST-LOG"
  printf '%s\n' '--------------------------------------------------------------'
}

print_env_row() {
  local env_name="$1"
  local log_path="${AOXC_DATA_ROOT}/logs/network/${env_name}/runtime.log"
  local home_path="${AOXC_DATA_ROOT}/home/${env_name}"
  local daemon_status='stopped'
  local genesis_status='missing'
  local accounts_status='missing'
  local last_log='<none>'
  local status_output=''

  status_output="$("${DAEMON_SCRIPT}" status "${env_name}" 2>/dev/null || true)"
  if [[ "${status_output}" == *"running"* ]]; then
    daemon_status="running"
  fi

  [[ -f "${home_path}/identity/genesis.json" ]] && genesis_status="present"
  [[ -f "${home_path}/identity/accounts.generated.json" ]] && accounts_status="present"

  if [[ -f "${log_path}" ]]; then
    last_log="$(tail -n 1 "${log_path}" 2>/dev/null | cut -c1-20)"
    [[ -n "${last_log}" ]] || last_log="<empty>"
  fi

  printf '%-10s | %-8s | %-10s | %-10s | %-20s\n' \
    "${env_name}" \
    "${daemon_status}" \
    "${genesis_status}" \
    "${accounts_status}" \
    "${last_log}"
}

print_dashboard() {
  print_dashboard_header
  print_env_row "mainnet"
  print_env_row "testnet"
  print_env_row "devnet"
  printf '%s\n' '--------------------------------------------------------------'
  printf 'Quick actions:\n'
  printf '  - Full auto flow  : ./scripts/aoxc_easy.sh flow devnet\n'
  printf '  - Environment run : ./scripts/aoxc_easy.sh auto-start\n'
  printf '  - Make wrapper    : ./scripts/aoxc_easy.sh makectl start testnet\n'
  printf '==============================================================\n'
}

ensure_binary_or_build() {
  if resolve_bin_path >/dev/null 2>&1; then
    return 0
  fi

  log_info "AOXC binary is missing. Executing: make package-bin"
  (
    cd "${ROOT_DIR}"
    make package-bin
  )
}

run_full_auto_flow() {
  local env_name=''

  env_name="$(resolve_env_or_default "${1:-}")"

  log_info "Starting full auto flow for environment '${env_name}'."
  run_doctor || true
  ensure_binary_or_build
  "${DAEMON_SCRIPT}" start "${env_name}"
  "${DAEMON_SCRIPT}" status "${env_name}"

  log_info "Full auto flow completed for environment '${env_name}'."
  print_dashboard
}

run_makectl() {
  local action="${1:-}"
  local env_name=''
  local target=''

  env_name="$(resolve_env_or_default "${2:-}")"

  case "${action}" in
    start|once|stop|status|restart)
      target="ops-${action}-${env_name}"
      ;;
    *)
      die "Invalid makectl action: ${action:-<empty>}. Use start, once, stop, status, or restart." 2
      ;;
  esac

  log_info "Executing Make target: ${target}"
  (
    cd "${ROOT_DIR}"
    make "${target}"
  )
}

print_menu() {
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
}

main() {
  local command="${1:-help}"
  local env_name="${2:-}"

  require_executable_script "${DAEMON_SCRIPT}"

  case "${command}" in
    help|-h|--help)
      print_usage
      ;;
    doctor)
      run_doctor
      ;;
    start|once|stop|status)
      env_name="$(resolve_env_or_default "${env_name}")"
      log_info "Executing command='${command}' for environment='${env_name}'."
      "${DAEMON_SCRIPT}" "${command}" "${env_name}"
      ;;
    restart)
      env_name="$(resolve_env_or_default "${env_name}")"
      log_info "Restarting environment '${env_name}'."
      "${DAEMON_SCRIPT}" stop "${env_name}"
      "${DAEMON_SCRIPT}" start "${env_name}"
      ;;
    logs)
      env_name="$(resolve_env_or_default "${env_name}")"
      log_info "Following runtime logs for environment '${env_name}'."
      exec tail -n 120 -f "${AOXC_DATA_ROOT}/logs/network/${env_name}/runtime.log"
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
    restart-dual)
      run_dual restart
      ;;
    auto-start)
      env_name="$(resolve_env_or_default "${AOXC_ENV:-}")"
      log_info "Auto-start selected environment '${env_name}'."
      "${DAEMON_SCRIPT}" start "${env_name}"
      ;;
    auto-once)
      env_name="$(resolve_env_or_default "${AOXC_ENV:-}")"
      log_info "Auto-once selected environment '${env_name}'."
      "${DAEMON_SCRIPT}" once "${env_name}"
      ;;
    dashboard)
      print_dashboard
      ;;
    flow)
      run_full_auto_flow "${env_name}"
      ;;
    makectl)
      run_makectl "${2:-}" "${3:-}"
      ;;
    menu)
      print_menu
      ;;
    *)
      print_usage >&2
      die "Unknown command: ${command}" 2
      ;;
  esac
}

main "$@"
