#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Provide single-runtime AOXC daemon lifecycle control for the canonical
#   operator surface.
#
# Operational Model:
#   - Resolve the AOXC binary from approved local locations
#   - Materialize one runtime root and one log root
#   - Support `start`, `once`, `status`, `stop`, and `tail`
#   - Persist PID-based lifecycle state for managed background execution
#   - Fail closed on invalid prerequisites or runtime drift
#
# Exit Codes:
#   0  Successful completion
#   2  Invalid invocation
#   4  AOXC binary resolution failure
#   5  Unsupported command
#   6  Invalid configuration
#   7  Bootstrap failure
#   8  Managed daemon start failure
#   9  Managed daemon stop failure
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly DEFAULT_NETWORK_TIMEOUT_MS=3000
readonly DEFAULT_DAEMON_SLEEP_SECS=2

readonly COMMAND="${1:-}"

AOXC_ROOT="${AOXC_ROOT:-${HOME}/.aoxc}"
AOXC_RUNTIME_ROOT="${AOXC_RUNTIME_ROOT:-${AOXC_ROOT}/runtime}"
AOXC_LOG_DIR="${AOXC_LOG_DIR:-${AOXC_ROOT}/logs}"
AOXC_BIN_PATH_OVERRIDE="${BIN_PATH:-}"

PID_FILE="${AOXC_LOG_DIR}/runtime.pid"
RUNTIME_LOG="${AOXC_LOG_DIR}/runtime.log"
BOOTSTRAP_MARKER="${AOXC_RUNTIME_ROOT}/.bootstrap_done"

log_info() {
  printf '[runtime-daemon][info] %s\n' "$*"
}

log_warn() {
  printf '[runtime-daemon][warn] %s\n' "$*" >&2
}

log_error() {
  printf '[runtime-daemon][error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"
  log_error "${message}"
  exit "${exit_code}"
}

print_usage() {
  cat <<'EOF'
Usage:
  ./scripts/runtime_daemon.sh <start|once|status|stop|tail>
EOF
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 6
}

ensure_directory() {
  local dir_path="$1"

  if [[ -e "${dir_path}" && ! -d "${dir_path}" ]]; then
    die "Path exists but is not a directory: ${dir_path}" 6
  fi

  mkdir -p "${dir_path}"
}

validate_non_negative_integer() {
  local value="$1"
  local name="$2"

  [[ "${value}" =~ ^[0-9]+$ ]] || die "Invalid value for ${name}: '${value}'. A non-negative integer is required." 6
}

resolve_bin_path() {
  if [[ -n "${AOXC_BIN_PATH_OVERRIDE}" ]]; then
    [[ -x "${AOXC_BIN_PATH_OVERRIDE}" ]] || die "BIN_PATH is set but not executable: ${AOXC_BIN_PATH_OVERRIDE}" 4
    printf '%s\n' "${AOXC_BIN_PATH_OVERRIDE}"
    return 0
  fi

  if [[ -x "${AOXC_ROOT}/bin/current/aoxc" ]]; then
    printf '%s\n' "${AOXC_ROOT}/bin/current/aoxc"
    return 0
  fi

  if [[ -x "${ROOT_DIR}/target/release/aoxc" ]]; then
    printf '%s\n' "${ROOT_DIR}/target/release/aoxc"
    return 0
  fi

  if [[ -x "${ROOT_DIR}/bin/aoxc" ]]; then
    printf '%s\n' "${ROOT_DIR}/bin/aoxc"
    return 0
  fi

  return 1
}

initialize_runtime_paths() {
  ensure_directory "${AOXC_ROOT}"
  ensure_directory "${AOXC_RUNTIME_ROOT}"
  ensure_directory "${AOXC_LOG_DIR}"
  touch "${RUNTIME_LOG}"
}

run_and_tee() {
  local log_file="$1"
  shift

  set +e
  "$@" 2>&1 | tee -a "${log_file}"
  local cmd_exit=${PIPESTATUS[0]}
  set -e

  return "${cmd_exit}"
}

bootstrap_runtime() {
  local bin_path="$1"

  export AOXC_HOME="${AOXC_RUNTIME_ROOT}"

  if [[ -f "${BOOTSTRAP_MARKER}" ]]; then
    log_info "Bootstrap already completed for runtime root '${AOXC_RUNTIME_ROOT}'."
    return 0
  fi

  log_info "Bootstrap started for runtime root '${AOXC_RUNTIME_ROOT}'."

  if ! run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" db-init --backend redb --format json; then
    die "db-init failed during runtime bootstrap." 7
  fi

  touch "${BOOTSTRAP_MARKER}"
  log_info "Bootstrap completed for runtime root '${AOXC_RUNTIME_ROOT}'."
}

run_once() {
  local bin_path="$1"

  export AOXC_HOME="${AOXC_RUNTIME_ROOT}"

  run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" produce-once --tx "AOXC_RUNTIME_$(date +%s)"

  run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" network-smoke \
      --timeout-ms "${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}" \
      --bind-host 127.0.0.1 \
      --port 0 \
      --payload "HEALTH_RUNTIME"
}

is_managed_pid_running() {
  local pid="$1"
  kill -0 "${pid}" >/dev/null 2>&1
}

start_daemon() {
  local bin_path="$1"
  local daemon_pid=''

  if [[ -f "${PID_FILE}" ]]; then
    daemon_pid="$(cat "${PID_FILE}")"
    if [[ "${daemon_pid}" =~ ^[0-9]+$ ]] && is_managed_pid_running "${daemon_pid}"; then
      log_info "Runtime is already running with PID ${daemon_pid}."
      return 0
    fi
    rm -f "${PID_FILE}"
  fi

  bootstrap_runtime "${bin_path}"

  (
    export AOXC_HOME="${AOXC_RUNTIME_ROOT}"

    while true; do
      "${bin_path}" produce-once --tx "AOXC_RUNTIME_DAEMON_$(date +%s)" >> "${RUNTIME_LOG}" 2>&1
      "${bin_path}" network-smoke \
        --timeout-ms "${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}" \
        --bind-host 127.0.0.1 \
        --port 0 \
        --payload "HEALTH_RUNTIME" >> "${RUNTIME_LOG}" 2>&1
      sleep "${DAEMON_SLEEP_SECS:-$DEFAULT_DAEMON_SLEEP_SECS}"
    done
  ) &

  daemon_pid="$!"
  printf '%s\n' "${daemon_pid}" > "${PID_FILE}"

  if ! is_managed_pid_running "${daemon_pid}"; then
    rm -f "${PID_FILE}"
    die "Managed daemon failed to start." 8
  fi

  log_info "Started runtime with PID ${daemon_pid}. Log: ${RUNTIME_LOG}"
}

status_daemon() {
  local daemon_pid=''

  if [[ -f "${PID_FILE}" ]]; then
    daemon_pid="$(cat "${PID_FILE}")"
    if [[ "${daemon_pid}" =~ ^[0-9]+$ ]] && is_managed_pid_running "${daemon_pid}"; then
      log_info "Runtime is running with PID ${daemon_pid}."
      return 0
    fi
  fi

  log_info "Runtime is stopped."
}

stop_daemon() {
  local daemon_pid=''

  if [[ ! -f "${PID_FILE}" ]]; then
    log_info "No PID file exists for runtime."
    return 0
  fi

  daemon_pid="$(cat "${PID_FILE}")"

  if [[ ! "${daemon_pid}" =~ ^[0-9]+$ ]]; then
    rm -f "${PID_FILE}"
    die "PID file contains an invalid process identifier." 9
  fi

  if ! is_managed_pid_running "${daemon_pid}"; then
    rm -f "${PID_FILE}"
    log_info "Managed process is no longer running. Stale PID file removed."
    return 0
  fi

  kill "${daemon_pid}" >/dev/null 2>&1 || die "Failed to send termination signal to PID ${daemon_pid}." 9

  local wait_round=0
  while is_managed_pid_running "${daemon_pid}"; do
    wait_round=$((wait_round + 1))
    if (( wait_round >= 5 )); then
      die "Managed process did not terminate within the expected interval." 9
    fi
    sleep 1
  done

  rm -f "${PID_FILE}"
  log_info "Stopped runtime with PID ${daemon_pid}."
}

tail_logs() {
  touch "${RUNTIME_LOG}"
  exec tail -n 100 -f "${RUNTIME_LOG}"
}

main() {
  local bin_path=''

  require_command sha256sum
  require_command awk
  require_command tail

  validate_non_negative_integer "${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}" "NETWORK_TIMEOUT_MS"
  validate_non_negative_integer "${DAEMON_SLEEP_SECS:-$DEFAULT_DAEMON_SLEEP_SECS}" "DAEMON_SLEEP_SECS"

  [[ -n "${COMMAND}" ]] || {
    print_usage >&2
    exit 2
  }

  initialize_runtime_paths

  if ! bin_path="$(resolve_bin_path)"; then
    die "AOXC binary not found. Run: make package-bin" 4
  fi

  case "${COMMAND}" in
    start)
      start_daemon "${bin_path}"
      ;;
    once)
      bootstrap_runtime "${bin_path}"
      run_once "${bin_path}"
      ;;
    status)
      status_daemon
      ;;
    stop)
      stop_daemon
      ;;
    tail)
      tail_logs
      ;;
    --help|-h|help)
      print_usage
      ;;
    *)
      die "Unknown command: ${COMMAND}" 5
      ;;
  esac
}

main "$@"
