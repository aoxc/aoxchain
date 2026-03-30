#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

# -----------------------------------------------------------------------------
# Purpose:
#   Provide a lightweight self-healing supervisor for non-containerized local
#   AOXC deployments.
#
# Operational Model:
#   - Resolve the AOXC binary from an approved set of locations.
#   - Execute a continuous "produce-once" loop.
#   - Restart the loop when it fails unexpectedly.
#   - Stop after a bounded number of restart attempts.
#
# Non-Goals:
#   - This script is not a cluster orchestrator.
#   - This script does not provide distributed failover semantics.
#   - This script does not daemonize itself.
#
# Exit Codes:
#   0  Supervisor terminated after a normal child exit
#   1  Restart threshold reached or supervisor-level runtime failure
#   2  Invalid configuration or binary resolution failure
#   3  Interrupted by operator or external termination signal
# -----------------------------------------------------------------------------

readonly DEFAULT_MAX_RESTARTS=20
readonly DEFAULT_RESTART_DELAY_SECS=3
readonly DEFAULT_PRODUCE_INTERVAL_SECS=1

MAX_RESTARTS="${MAX_RESTARTS:-$DEFAULT_MAX_RESTARTS}"
RESTART_DELAY_SECS="${RESTART_DELAY_SECS:-$DEFAULT_RESTART_DELAY_SECS}"
PRODUCE_INTERVAL_SECS="${PRODUCE_INTERVAL_SECS:-$DEFAULT_PRODUCE_INTERVAL_SECS}"

supervisor_should_stop=0

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
  # A termination signal is treated as an explicit operator or system request.
  # The supervisor does not attempt a restart in this path because doing so
  # would conflict with the caller's shutdown intent.
  supervisor_should_stop=1
  log_warn "Termination signal received. Supervisor will stop after the current control point."
}

trap on_termination_signal INT TERM

validate_non_negative_integer() {
  local value="$1"
  local name="$2"

  if [[ ! "${value}" =~ ^[0-9]+$ ]]; then
    die "Invalid value for ${name}: '${value}'. A non-negative integer is required." 2
  fi
}

resolve_bin_path() {
  # The resolution order is intentionally explicit to avoid ambiguous binary
  # selection and to preserve operator predictability across local environments.
  if [[ -n "${BIN_PATH:-}" ]]; then
    if [[ -x "${BIN_PATH}" ]]; then
      printf '%s\n' "${BIN_PATH}"
      return 0
    fi

    die "BIN_PATH is set but not executable: ${BIN_PATH}" 2
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

  log_info "Starting producer loop using binary: ${bin_path}"

  while true; do
    if (( supervisor_should_stop == 1 )); then
      log_warn "Supervisor stop requested before producer invocation."
      return 0
    fi

    # The transaction marker provides a minimal, operator-visible trace of the
    # supervised invocation sequence. This is useful during local diagnostics.
    if ! "${bin_path}" produce-once --tx "AOXC_SUPERVISOR_${iteration}"; then
      cmd_exit=$?
      log_warn "produce-once failed at iteration ${iteration} with exit code ${cmd_exit}."
      return "${cmd_exit}"
    fi

    iteration=$((iteration + 1))

    if (( supervisor_should_stop == 1 )); then
      log_warn "Supervisor stop requested after producer invocation."
      return 0
    fi

    sleep "${PRODUCE_INTERVAL_SECS}"
  done
}

main() {
  local bin_path
  local restart_count=0
  local loop_exit_code=0

  validate_configuration

  if ! bin_path="$(resolve_bin_path)"; then
    die "Unable to locate an executable AOXC binary. Build or install it with: make package-bin" 2
  fi

  log_info "Supervisor initialized."
  log_info "Resolved binary path: ${bin_path}"
  log_info "Configured maximum restarts: ${MAX_RESTARTS}"
  log_info "Configured restart delay (seconds): ${RESTART_DELAY_SECS}"
  log_info "Configured produce interval (seconds): ${PRODUCE_INTERVAL_SECS}"

  while true; do
    if (( supervisor_should_stop == 1 )); then
      log_warn "Supervisor shutdown requested before launching producer loop."
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

    log_warn \
      "Producer loop exited with code ${loop_exit_code}. Restart attempt ${restart_count}/${MAX_RESTARTS} will occur after ${RESTART_DELAY_SECS} second(s)."

    if (( restart_count >= MAX_RESTARTS )); then
      die "Maximum restart threshold reached. Supervisor is stopping." 1
    fi

    sleep "${RESTART_DELAY_SECS}"
  done
}

main "$@"
