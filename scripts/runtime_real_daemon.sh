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
#   Run a deterministic local "real chain" workflow for AOXC in non-container
#   environments.
#
# Execution Model:
#   - Resolve the AOXC binary from approved local locations.
#   - Prepare runtime directories and logs.
#   - Bootstrap local key material, genesis state, and node home.
#   - Execute bounded or unbounded production cycles.
#   - Perform post-cycle network smoke checks.
#
# Security Notes:
#   - Secrets must not be hardcoded in the script body.
#   - Numeric runtime inputs are validated before execution begins.
#   - The script emits operator-visible logs for each critical control point.
#
# Exit Codes:
#   0  Successful completion
#   2  Invalid configuration or binary resolution failure
#   3  Interrupted by operator or external termination signal
# -----------------------------------------------------------------------------

readonly DEFAULT_AOXC_DATA_ROOT="${HOME}/.AOXCData"
readonly DEFAULT_MAX_CYCLES=0
readonly DEFAULT_ROUND_PER_CYCLE=60
readonly DEFAULT_SLEEP_MS=1000
readonly DEFAULT_NETWORK_ROUNDS=5
readonly DEFAULT_NETWORK_TIMEOUT_MS=3000
readonly DEFAULT_NETWORK_PAUSE_MS=250
readonly DEFAULT_KEY_PROFILE="testnet"
readonly DEFAULT_KEY_NAME="validator-real-01"
readonly DEFAULT_CHAIN_NUM=1001
readonly DEFAULT_BLOCK_TIME=6
readonly DEFAULT_TREASURY_AMOUNT=1000000000000

AOXC_DATA_ROOT="${AOXC_DATA_ROOT:-$DEFAULT_AOXC_DATA_ROOT}"
AOXC_HOME_DIR="${AOXC_HOME_DIR:-${AOXC_DATA_ROOT}/home/real}"
LOG_DIR="${LOG_DIR:-${AOXC_DATA_ROOT}/logs/real-chain}"

MAX_CYCLES="${MAX_CYCLES:-$DEFAULT_MAX_CYCLES}"
ROUND_PER_CYCLE="${ROUND_PER_CYCLE:-$DEFAULT_ROUND_PER_CYCLE}"
SLEEP_MS="${SLEEP_MS:-$DEFAULT_SLEEP_MS}"
NETWORK_ROUNDS="${NETWORK_ROUNDS:-$DEFAULT_NETWORK_ROUNDS}"
NETWORK_TIMEOUT_MS="${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}"
NETWORK_PAUSE_MS="${NETWORK_PAUSE_MS:-$DEFAULT_NETWORK_PAUSE_MS}"

KEY_PROFILE="${KEY_PROFILE:-$DEFAULT_KEY_PROFILE}"
KEY_NAME="${KEY_NAME:-$DEFAULT_KEY_NAME}"
AOXC_KEY_PASSWORD="${AOXC_KEY_PASSWORD:-}"

CHAIN_NUM="${CHAIN_NUM:-$DEFAULT_CHAIN_NUM}"
BLOCK_TIME="${BLOCK_TIME:-$DEFAULT_BLOCK_TIME}"
TREASURY_AMOUNT="${TREASURY_AMOUNT:-$DEFAULT_TREASURY_AMOUNT}"

RUNTIME_LOG=''
HEALTH_LOG=''
SHUTDOWN_REQUESTED=0

log_runtime() {
  printf '[real-chain] %s\n' "$*" | tee -a "${RUNTIME_LOG}"
}

log_runtime_warn() {
  printf '[real-chain][warn] %s\n' "$*" | tee -a "${RUNTIME_LOG}" >&2
}

log_runtime_error() {
  printf '[real-chain][error] %s\n' "$*" | tee -a "${RUNTIME_LOG}" >&2
}

log_health() {
  printf '[real-chain] %s\n' "$*" | tee -a "${HEALTH_LOG}"
}

log_health_warn() {
  printf '[real-chain][warn] %s\n' "$*" | tee -a "${HEALTH_LOG}" >&2
}

die() {
  local message="$1"
  local exit_code="$2"
  log_runtime_error "${message}"
  exit "${exit_code}"
}

on_termination_signal() {
  SHUTDOWN_REQUESTED=1

  if [[ -n "${RUNTIME_LOG}" ]]; then
    log_runtime_warn "Termination signal received. Shutdown will occur at the next safe control boundary."
  else
    printf '[real-chain][warn] Termination signal received before log initialization.\n' >&2
  fi
}

trap on_termination_signal INT TERM

validate_non_negative_integer() {
  local value="$1"
  local name="$2"

  if [[ ! "${value}" =~ ^[0-9]+$ ]]; then
    die "Invalid value for ${name}: '${value}'. A non-negative integer is required." 2
  fi
}

validate_positive_integer() {
  local value="$1"
  local name="$2"

  if [[ ! "${value}" =~ ^[1-9][0-9]*$ ]]; then
    die "Invalid value for ${name}: '${value}'. A positive integer is required." 2
  fi
}

resolve_bin_path() {
  # The binary resolution order is intentionally explicit to prevent ambiguous
  # execution across multiple local installation locations.
  if [[ -n "${BIN_PATH:-}" ]]; then
    if [[ -x "${BIN_PATH}" ]]; then
      printf '%s\n' "${BIN_PATH}"
      return 0
    fi

    return 1
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

ensure_directory() {
  local dir_path="$1"

  if [[ -e "${dir_path}" && ! -d "${dir_path}" ]]; then
    die "Path exists but is not a directory: ${dir_path}" 2
  fi

  mkdir -p "${dir_path}"
}

milliseconds_to_seconds_string() {
  local milliseconds="$1"
  local seconds_part=$((milliseconds / 1000))
  local millis_part=$((milliseconds % 1000))

  printf '%d.%03d' "${seconds_part}" "${millis_part}"
}

validate_configuration() {
  validate_non_negative_integer "${MAX_CYCLES}" "MAX_CYCLES"
  validate_positive_integer "${ROUND_PER_CYCLE}" "ROUND_PER_CYCLE"
  validate_non_negative_integer "${SLEEP_MS}" "SLEEP_MS"
  validate_positive_integer "${NETWORK_ROUNDS}" "NETWORK_ROUNDS"
  validate_positive_integer "${NETWORK_TIMEOUT_MS}" "NETWORK_TIMEOUT_MS"
  validate_non_negative_integer "${NETWORK_PAUSE_MS}" "NETWORK_PAUSE_MS"
  validate_positive_integer "${CHAIN_NUM}" "CHAIN_NUM"
  validate_positive_integer "${BLOCK_TIME}" "BLOCK_TIME"
  validate_non_negative_integer "${TREASURY_AMOUNT}" "TREASURY_AMOUNT"

  if [[ -z "${KEY_PROFILE}" ]]; then
    die "KEY_PROFILE must not be empty." 2
  fi

  if [[ -z "${KEY_NAME}" ]]; then
    die "KEY_NAME must not be empty." 2
  fi

  if [[ -z "${AOXC_KEY_PASSWORD}" ]]; then
    die "AOXC_KEY_PASSWORD must be provided via environment variable. Hardcoded credentials are not permitted." 2
  fi
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

bootstrap_key_material() {
  local bin_path="$1"

  log_runtime "key-bootstrap started (profile=${KEY_PROFILE}, name=${KEY_NAME})"

  if ! run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" key-bootstrap \
      --profile "${KEY_PROFILE}" \
      --name "${KEY_NAME}" \
      --password "${AOXC_KEY_PASSWORD}" \
      --home "${AOXC_HOME}"; then
    die "key-bootstrap failed." 2
  fi
}

bootstrap_genesis() {
  local bin_path="$1"

  log_runtime "genesis-init started"

  if ! run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" genesis-init \
      --home "${AOXC_HOME}" \
      --chain-num "${CHAIN_NUM}" \
      --block-time "${BLOCK_TIME}" \
      --treasury "${TREASURY_AMOUNT}"; then
    die "genesis-init failed." 2
  fi
}

bootstrap_node_home() {
  local bin_path="$1"

  log_runtime "node-bootstrap started"

  if ! run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" node-bootstrap \
      --home "${AOXC_HOME}"; then
    die "node-bootstrap failed." 2
  fi
}

run_production_cycle() {
  local bin_path="$1"
  local cycle="$2"
  local local_round=1
  local cmd_exit=0
  local tx_payload=''
  local cycle_timestamp=''

  cycle_timestamp="$(date -Iseconds)"
  log_runtime "[${cycle_timestamp}] cycle=${cycle} produce loop started"

  while (( local_round <= ROUND_PER_CYCLE )); do
    if (( SHUTDOWN_REQUESTED == 1 )); then
      log_runtime_warn "Shutdown requested before produce-once invocation (cycle=${cycle}, round=${local_round})."
      return 0
    fi

    tx_payload="AOXC_REAL_${cycle}_${local_round}_$(date +%s)"

    if ! run_and_tee "${RUNTIME_LOG}" \
      "${bin_path}" produce-once \
        --tx "${tx_payload}"; then
      cmd_exit=$?
      log_runtime_warn "cycle=${cycle} produce-once failed at round=${local_round} with exit code ${cmd_exit}."
      return "${cmd_exit}"
    fi

    if (( local_round < ROUND_PER_CYCLE )); then
      sleep "$(milliseconds_to_seconds_string "${SLEEP_MS}")"
    fi

    local_round=$((local_round + 1))
  done

  return 0
}

run_network_health_probe() {
  local bin_path="$1"
  local cycle="$2"
  local health_round=1
  local cmd_exit=0
  local cycle_timestamp=''

  cycle_timestamp="$(date -Iseconds)"
  log_health "[${cycle_timestamp}] cycle=${cycle} network and health probe started"

  while (( health_round <= NETWORK_ROUNDS )); do
    if (( SHUTDOWN_REQUESTED == 1 )); then
      log_health_warn "Shutdown requested before network-smoke invocation (cycle=${cycle}, round=${health_round})."
      return 0
    fi

    if ! run_and_tee "${HEALTH_LOG}" \
      "${bin_path}" network-smoke \
        --timeout-ms "${NETWORK_TIMEOUT_MS}" \
        --bind-host 127.0.0.1 \
        --port 0 \
        --payload "AOXC_HEALTH_CYCLE_${cycle}_${health_round}"; then
      cmd_exit=$?
      log_health_warn "cycle=${cycle} network-smoke failed at round=${health_round} with exit code ${cmd_exit}."
      return "${cmd_exit}"
    fi

    if (( health_round < NETWORK_ROUNDS )); then
      sleep "$(milliseconds_to_seconds_string "${NETWORK_PAUSE_MS}")"
    fi

    health_round=$((health_round + 1))
  done

  return 0
}

main() {
  local bin_path=''
  local cycle=0
  local node_exit=0
  local network_exit=0

  validate_configuration

  if ! bin_path="$(resolve_bin_path)"; then
    printf '[real-chain][error] binary is not executable: %s\n' "${BIN_PATH:-}" >&2
    printf '[real-chain][hint] run: make package-bin (installs $HOME/.AOXCData/bin/aoxc)\n' >&2
    exit 2
  fi

  ensure_directory "${AOXC_DATA_ROOT}"
  ensure_directory "${AOXC_HOME_DIR}"
  ensure_directory "${LOG_DIR}"

  RUNTIME_LOG="${LOG_DIR}/runtime.log"
  HEALTH_LOG="${LOG_DIR}/health.log"

  touch "${RUNTIME_LOG}" "${HEALTH_LOG}"

  export AOXC_HOME="${AOXC_HOME_DIR}"

  log_runtime "===== startup ====="
  log_runtime "bin=${bin_path}"
  log_runtime "AOXC_HOME=${AOXC_HOME}"
  log_runtime "MAX_CYCLES=${MAX_CYCLES}"
  log_runtime "ROUND_PER_CYCLE=${ROUND_PER_CYCLE}"
  log_runtime "SLEEP_MS=${SLEEP_MS}"
  log_runtime "NETWORK_ROUNDS=${NETWORK_ROUNDS}"
  log_runtime "NETWORK_TIMEOUT_MS=${NETWORK_TIMEOUT_MS}"
  log_runtime "NETWORK_PAUSE_MS=${NETWORK_PAUSE_MS}"

  bootstrap_key_material "${bin_path}"
  bootstrap_genesis "${bin_path}"
  bootstrap_node_home "${bin_path}"

  while true; do
    if (( SHUTDOWN_REQUESTED == 1 )); then
      log_runtime_warn "Shutdown requested before cycle start. Exiting gracefully."
      exit 3
    fi

    cycle=$((cycle + 1))
    node_exit=0
    network_exit=0

    if ! run_production_cycle "${bin_path}" "${cycle}"; then
      node_exit=$?
    fi

    if ! run_network_health_probe "${bin_path}" "${cycle}"; then
      network_exit=$?
    fi

    if (( node_exit != 0 || network_exit != 0 )); then
      log_runtime_warn "cycle=${cycle} completed with node_exit=${node_exit} network_exit=${network_exit}."
      log_health_warn "cycle=${cycle} completed with node_exit=${node_exit} network_exit=${network_exit}."
    else
      log_runtime "cycle=${cycle} node and network checks passed."
      log_health "cycle=${cycle} node and network checks passed."
    fi

    if (( MAX_CYCLES > 0 && cycle >= MAX_CYCLES )); then
      log_runtime "Maximum cycle threshold reached (${MAX_CYCLES}). Exiting normally."
      break
    fi
  done
}

main "$@"
