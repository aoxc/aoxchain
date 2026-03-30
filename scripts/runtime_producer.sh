#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Execute a continuous AOXC producer loop for local and operator-driven
#   runtime workflows.
#
# Operational Model:
#   - Resolve the AOXC binary from approved local locations
#   - Emit repeated `produce-once` calls with traceable transaction payloads
#   - Persist operator-visible runtime logs
#   - Support bounded or unbounded execution through MAX_ROUNDS
#
# Exit Codes:
#   0  Successful completion
#   2  AOXC binary resolution failure
#   3  Invalid runtime configuration
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly DEFAULT_AOXC_DATA_ROOT="${HOME}/.AOXCData"
readonly DEFAULT_TX_PREFIX="auto-tx"
readonly DEFAULT_SLEEP_SECS=2
readonly DEFAULT_MAX_ROUNDS=0

AOXC_DATA_ROOT="${AOXC_DATA_ROOT:-$DEFAULT_AOXC_DATA_ROOT}"
TX_PREFIX="${TX_PREFIX:-$DEFAULT_TX_PREFIX}"
SLEEP_SECS="${SLEEP_SECS:-$DEFAULT_SLEEP_SECS}"
MAX_ROUNDS="${MAX_ROUNDS:-$DEFAULT_MAX_ROUNDS}"
LOG_FILE="${LOG_FILE:-${AOXC_DATA_ROOT}/logs/continuous-producer.log}"

log_info() {
  printf '[producer][info] %s\n' "$*"
}

log_warn() {
  printf '[producer][warn] %s\n' "$*" >&2
}

log_error() {
  printf '[producer][error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"

  log_error "${message}"
  exit "${exit_code}"
}

validate_non_negative_integer() {
  local value="$1"
  local name="$2"

  [[ "${value}" =~ ^[0-9]+$ ]] || die "Invalid value for ${name}: '${value}'. A non-negative integer is required." 3
}

resolve_bin_path() {
  # The resolution order is intentionally explicit to prevent ambiguous runtime
  # selection across packaged and repository-local binary locations.
  if [[ -n "${BIN_PATH:-}" ]]; then
    [[ -x "${BIN_PATH}" ]] || die "BIN_PATH is set but not executable: ${BIN_PATH}" 2
    printf '%s\n' "${BIN_PATH}"
    return 0
  fi

  if [[ -x "${HOME}/.AOXCData/bin/aoxc" ]]; then
    printf '%s\n' "${HOME}/.AOXCData/bin/aoxc"
    return 0
  fi

  if [[ -x "./bin/aoxc" ]]; then
    printf '%s\n' "./bin/aoxc"
    return 0
  fi

  return 1
}

ensure_log_directory() {
  local log_dir=''

  log_dir="$(dirname "${LOG_FILE}")"

  if [[ -e "${log_dir}" && ! -d "${log_dir}" ]]; then
    die "Log directory path exists but is not a directory: ${log_dir}" 3
  fi

  mkdir -p "${log_dir}"
}

run_once_with_logging() {
  local bin_path="$1"
  local tx_payload="$2"

  set +e
  "${bin_path}" produce-once --tx "${tx_payload}" 2>&1 | tee -a "${LOG_FILE}"
  local exit_code=${PIPESTATUS[0]}
  set -e

  return "${exit_code}"
}

main() {
  local bin_path=''
  local round=0
  local tx_payload=''
  local command_exit=0

  validate_non_negative_integer "${SLEEP_SECS}" "SLEEP_SECS"
  validate_non_negative_integer "${MAX_ROUNDS}" "MAX_ROUNDS"

  if ! bin_path="$(resolve_bin_path)"; then
    die "Unable to locate an executable AOXC binary. Build or install it with: make package-bin" 2
  fi

  ensure_log_directory

  log_info "Continuous producer loop initialized." | tee -a "${LOG_FILE}"
  log_info "Resolved binary path: ${bin_path}" | tee -a "${LOG_FILE}"
  log_info "Configured sleep interval (seconds): ${SLEEP_SECS}" | tee -a "${LOG_FILE}"
  log_info "Configured maximum rounds: ${MAX_ROUNDS}" | tee -a "${LOG_FILE}"

  while true; do
    round=$((round + 1))
    tx_payload="${TX_PREFIX}-${round}-$(date +%s)"

    log_info "Round ${round}: produce-once tx=${tx_payload}" | tee -a "${LOG_FILE}"

    if ! run_once_with_logging "${bin_path}" "${tx_payload}"; then
      command_exit=$?
      log_warn "Round ${round}: produce-once failed with exit code ${command_exit}" | tee -a "${LOG_FILE}" >&2
    else
      log_info "Round ${round}: produce-once completed successfully" | tee -a "${LOG_FILE}"
    fi

    if (( MAX_ROUNDS > 0 && round >= MAX_ROUNDS )); then
      log_info "Maximum round threshold reached: ${MAX_ROUNDS}" | tee -a "${LOG_FILE}"
      break
    fi

    sleep "${SLEEP_SECS}"
  done
}

main "$@"
