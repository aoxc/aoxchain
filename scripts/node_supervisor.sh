#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Provide a lightweight self-healing supervisor for non-containerized local
#   AOXC deployments.
#
# Operational Model:
#   - Resolve the AOXC runtime binary from approved local locations
#   - Execute a continuous producer loop through repeated `produce-once` calls
#   - Restart the loop when an unexpected failure occurs
#   - Stop after a bounded number of restart attempts
#
# Exit Codes:
#   0  Successful completion
#   1  Restart threshold reached or supervisor-level operational failure
#   2  Invalid configuration or AOXC binary resolution failure
#   3  Interrupted by operator or external termination signal
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly DEFAULT_MAX_RESTARTS=20
readonly DEFAULT_RESTART_DELAY_SECS=3
readonly DEFAULT_PRODUCE_INTERVAL_SECS=1

MAX_RESTARTS="${MAX_RESTARTS:-$DEFAULT_MAX_RESTARTS}"
RESTART_DELAY_SECS="${RESTART_DELAY_SECS:-$DEFAULT_RESTART_DELAY_SECS}"
PRODUCE_INTERVAL_SECS="${PRODUCE_INTERVAL_SECS:-$DEFAULT_PRODUCE_INTERVAL_SECS}"

SUPERVISOR_STOP_REQUESTED=0

log_info() {
  printf '[info] %s\n' "$*"
}

log_warn() {
  printf '[warn] %s\n' "$*" >&2
}

log_error() {
  printf '[error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"

  log_error "${message}"
  exit "${exit_code}"
}

on_termination_signal() {
  # A termination signal is treated as an explicit operator or host-level
  # shutdown request. The supervisor therefore stops at the next safe control
  # boundary rather than attempting another restart cycle.
  SUPERVISOR_STOP_REQUESTED=1
  log_warn "Termination signal received. Supervisor shutdown has been requested."
}

trap on_termination_signal INT TERM

validate_non_negative_integer() {
  local value="$1"
  local name="$2"

  [[ "${value}" =~ ^[0-9]+$ ]] || die "Invalid value for ${name}: '${value}'. A non-negative integer is required." 2
}

resolve_bin_path() {
  # The resolution order is intentionally explicit to prevent ambiguous binary
  # selection across packaged and repository-local runtime surfaces.
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

validate_configuration() {
  validate_non_negative_integer "${MAX_RESTARTS}" "MAX_RESTARTS"
  validate_non_negative_integer "${RESTART_DELAY_SECS}" "RESTART_DELAY_SECS"
  validate_non_negative_integer "${PRODUCE_INTERVAL_SECS}" "PRODUCE_INTERVAL_SECS"
}

run_producer_loop() {
  local bin_path="$1"
  local iteration=1
  local cmd_exit=0

  log_info "Starting supervised producer loop using binary: ${bin_path}"

  while true; do
    if (( SUPERVISOR_STOP_REQUESTED == 1 )); then
      log_warn "Supervisor stop was requested before the next producer invocation."
      return 0
    fi

    if ! "${bin_path}" produce-once --tx "AOXC_SUPERVISOR_${iteration}"; then
      cmd_exit=$?
      log_warn "produce-once failed at iteration ${iteration} with exit code ${cmd_exit}."
      return "${cmd_exit}"
    fi

    iteration=$((iteration + 1))

    if (( SUPERVISOR_STOP_REQUESTED == 1 )); then
      log_warn "Supervisor stop was requested after producer invocation."
      return 0
    fi

    sleep "${PRODUCE_INTERVAL_SECS}"
  done
}

main() {
  local bin_path=''
  local restart_count=0
  local loop_exit_code=0

  validate_configuration

  if ! bin_path="$(resolve_bin_path)"; then
    die "Unable to locate an executable AOXC binary. Build or install it with: make package-bin" 2
  fi

  log_info "AOXC local supervisor initialized."
  log_info "Resolved binary path: ${bin_path}"
  log_info "Configured maximum restarts: ${MAX_RESTARTS}"
  log_info "Configured restart delay (seconds): ${RESTART_DELAY_SECS}"
  log_info "Configured produce interval (seconds): ${PRODUCE_INTERVAL_SECS}"

  while true; do
    if (( SUPERVISOR_STOP_REQUESTED == 1 )); then
      log_warn "Supervisor shutdown requested before launching the producer loop."
      exit 3
    fi

    set +e
    run_producer_loop "${bin_path}"
    loop_exit_code=$?
    set -e

    if (( loop_exit_code == 0 )); then
      log_info "Producer loop exited normally. Supervisor will stop."
      exit 0
    fi

    restart_count=$((restart_count + 1))

    log_warn "Producer loop exited with code ${loop_exit_code}. Restart attempt ${restart_count}/${MAX_RESTARTS} will occur after ${RESTART_DELAY_SECS} second(s)."

    if (( restart_count >= MAX_RESTARTS )); then
      die "Maximum restart threshold reached. Supervisor is stopping." 1
    fi

    sleep "${RESTART_DELAY_SECS}"
  done
}

main "$@"
