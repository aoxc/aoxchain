#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Provide environment-scoped AOXC daemon lifecycle control for the canonical
#   `mainnet`, `testnet`, and `devnet` runtime surfaces.
#
# Operational Model:
#   - Resolve the AOXC binary from approved local locations
#   - Materialize environment-scoped runtime home and log paths
#   - Bootstrap the selected environment when required
#   - Support `start`, `once`, `status`, `stop`, and `tail` control flows
#   - Persist PID-based lifecycle state for managed background execution
#
# Security and Reliability Notes:
#   - Secrets must not be hardcoded in the script body
#   - Runtime state must remain isolated per environment
#   - PID lifecycle control must avoid unrelated process termination
#   - Environment bootstrap must fail closed on invalid prerequisites
#
# Exit Codes:
#   0  Successful completion
#   2  Invalid invocation
#   3  Unsupported or invalid environment
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
readonly DEFAULT_GENESIS_ACCOUNT_COUNT=5

COMMAND="${1:-}"
TARGET_ENV="${2:-devnet}"

AOXC_DATA_ROOT="${AOXC_DATA_ROOT:-${HOME}/.AOXCData}"
AOXC_HOME_DIR="${AOXC_HOME_DIR:-${AOXC_DATA_ROOT}/home/${TARGET_ENV}}"
LOG_DIR="${LOG_DIR:-${AOXC_DATA_ROOT}/logs/network/${TARGET_ENV}}"
PID_FILE=''
RUNTIME_LOG=''

KEY_PROFILE=''
VALIDATOR_NAME=''
CHAIN_NUM=''
BLOCK_TIME=''
TREASURY=''

log_info() {
  printf '[network-env][info] %s\n' "$*"
}

log_warn() {
  printf '[network-env][warn] %s\n' "$*" >&2
}

log_error() {
  printf '[network-env][error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"

  log_error "${message}"
  exit "${exit_code}"
}

print_usage() {
  cat <<EOF
Usage:
  $(basename "$0") <start|once|status|stop|tail> [mainnet|testnet|devnet]
EOF
}

validate_non_negative_integer() {
  local value="$1"
  local name="$2"

  [[ "${value}" =~ ^[0-9]+$ ]] || die "Invalid value for ${name}: '${value}'. A non-negative integer is required." 6
}

resolve_bin_path() {
  # The binary resolution order is intentionally explicit to prevent ambiguous
  # execution across packaged and repository-local runtime layouts.
  if [[ -n "${BIN_PATH:-}" ]]; then
    [[ -x "${BIN_PATH}" ]] || die "BIN_PATH is set but not executable: ${BIN_PATH}" 4
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

run_and_tee() {
  local log_file="$1"
  shift

  set +e
  "$@" 2>&1 | tee -a "${log_file}"
  local cmd_exit=${PIPESTATUS[0]}
  set -e

  return "${cmd_exit}"
}

resolve_default_root_seed() {
  local env="$1"
  local seed_file="${AOXC_DATA_ROOT}/seeds/${env}.root.seed"
  local generated_seed=''

  ensure_directory "$(dirname "${seed_file}")"

  if [[ -s "${seed_file}" ]]; then
    tr -d '\r\n' < "${seed_file}"
    return 0
  fi

  generated_seed="$(printf 'AOXC::ROOT::%s::%s::%s' \
    "${env}" \
    "$(hostname -s 2>/dev/null || printf 'unknown-host')" \
    "$(date -u +%s)" | sha256sum | awk '{print $1}')"

  printf '%s\n' "${generated_seed}" > "${seed_file}"
  chmod 600 "${seed_file}"

  log_info "Generated root seed for environment '${env}' at '${seed_file}'."
  printf '%s' "${generated_seed}"
}

derive_account_id() {
  local root_seed="$1"
  local env="$2"
  local index="$3"

  printf 'AOXC_%s_%s' \
    "${env^^}" \
    "$(printf '%s::%s::%s' "${root_seed}" "${env}" "${index}" | sha256sum | awk '{print substr($1,1,24)}')"
}

materialize_generated_accounts() {
  local env="$1"
  local root_seed="$2"
  local desired_count="$3"
  local accounts_file="${AOXC_HOME_DIR}/identity/accounts.generated.json"
  local ledger_file="${AOXC_HOME_DIR}/support/generated-accounts.json"
  local i=0
  local account_id=''
  local account_seed=''

  ensure_directory "${AOXC_HOME_DIR}/identity"
  ensure_directory "${AOXC_HOME_DIR}/support"

  {
    printf '{\n'
    printf '  "schema_version": 1,\n'
    printf '  "environment": "%s",\n' "${env}"
    printf '  "root_seed_sha256": "%s",\n' "$(printf '%s' "${root_seed}" | sha256sum | awk '{print $1}')"
    printf '  "accounts": [\n'

    for (( i = 1; i <= desired_count; i++ )); do
      account_id="$(derive_account_id "${root_seed}" "${env}" "${i}")"
      account_seed="$(printf '%s::%s::%s::seed' "${root_seed}" "${env}" "${i}" | sha256sum | awk '{print $1}')"

      printf '    {\n'
      printf '      "index": %d,\n' "${i}"
      printf '      "name": "validator-auto-%d",\n' "${i}"
      printf '      "account_id": "%s",\n' "${account_id}"
      printf '      "seed_material_sha256": "%s",\n' "${account_seed}"
      printf '      "initial_balance": "1000000000"\n'
      printf '    }'

      if (( i < desired_count )); then
        printf ','
      fi
      printf '\n'
    done

    printf '  ]\n'
    printf '}\n'
  } > "${accounts_file}"

  cp "${accounts_file}" "${ledger_file}"
  log_info "Generated ${desired_count} deterministic accounts for environment '${env}'."
}

patch_genesis_with_generated_accounts() {
  local env="$1"
  local genesis_file="${AOXC_HOME_DIR}/identity/genesis.json"
  local accounts_file="${AOXC_HOME_DIR}/identity/accounts.generated.json"

  if [[ ! -f "${genesis_file}" || ! -f "${accounts_file}" ]]; then
    log_warn "Genesis or generated accounts artifact is missing. Genesis patch will be skipped."
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

generated_accounts = []
for row in accounts_doc.get("accounts", []):
    generated_accounts.append(
        {
            "account_id": row["account_id"],
            "balance": str(row.get("initial_balance", "1000000000")),
            "role": f"generated-validator-{row.get('index', 0)}",
        }
    )

state = genesis.setdefault("state", {})
if generated_accounts:
    state["accounts"] = generated_accounts

bindings = genesis.setdefault("bindings", {})
bindings["accounts_file"] = "accounts.generated.json"

metadata = genesis.setdefault("metadata", {})
metadata["description"] = (
    f"Canonical AOXC {env} genesis configuration with deterministic generated accounts."
)

genesis_path.write_text(json.dumps(genesis, indent=2) + "\n", encoding="utf-8")
PY

  log_info "Patched genesis with deterministic generated accounts for environment '${env}'."
}

configure_environment_profile() {
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
    *)
      die "Invalid environment: ${TARGET_ENV}" 3
      ;;
  esac
}

validate_configuration() {
  validate_non_negative_integer "${GENESIS_ACCOUNT_COUNT:-$DEFAULT_GENESIS_ACCOUNT_COUNT}" "GENESIS_ACCOUNT_COUNT"
  validate_non_negative_integer "${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}" "NETWORK_TIMEOUT_MS"
  validate_non_negative_integer "${DAEMON_SLEEP_SECS:-$DEFAULT_DAEMON_SLEEP_SECS}" "DAEMON_SLEEP_SECS"

  [[ -n "${COMMAND}" ]] || {
    print_usage >&2
    exit 2
  }

  case "${TARGET_ENV}" in
    mainnet|testnet|devnet) ;;
    *)
      die "Invalid environment: ${TARGET_ENV}" 3
      ;;
  esac

  if [[ -n "${AOXC_KEY_PASSWORD:-}" ]]; then
    :
  else
    die "AOXC_KEY_PASSWORD must be provided via environment variable. Hardcoded credentials are not permitted." 6
  fi
}

initialize_runtime_paths() {
  PID_FILE="${LOG_DIR}/daemon.pid"
  RUNTIME_LOG="${LOG_DIR}/runtime.log"

  ensure_directory "${AOXC_DATA_ROOT}"
  ensure_directory "${AOXC_HOME_DIR}"
  ensure_directory "${LOG_DIR}"

  touch "${RUNTIME_LOG}"
}

bootstrap_env() {
  local bin_path="$1"
  local root_seed=''
  local account_count="${GENESIS_ACCOUNT_COUNT:-$DEFAULT_GENESIS_ACCOUNT_COUNT}"

  export AOXC_HOME="${AOXC_HOME_DIR}"

  if [[ -f "${AOXC_HOME_DIR}/.bootstrap_done" ]]; then
    log_info "Bootstrap already completed for environment '${TARGET_ENV}'."
    return 0
  fi

  log_info "Bootstrap started for environment '${TARGET_ENV}'."

  root_seed="$(resolve_default_root_seed "${TARGET_ENV}")"
  materialize_generated_accounts "${TARGET_ENV}" "${root_seed}" "${account_count}"

  if ! run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" key-bootstrap \
      --profile "${KEY_PROFILE}" \
      --name "${VALIDATOR_NAME}" \
      --password "${AOXC_KEY_PASSWORD}" \
      --home "${AOXC_HOME_DIR}"; then
    die "key-bootstrap failed for environment '${TARGET_ENV}'." 7
  fi

  if ! run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" genesis-init \
      --home "${AOXC_HOME_DIR}" \
      --chain-num "${CHAIN_NUM}" \
      --block-time "${BLOCK_TIME}" \
      --treasury "${TREASURY}"; then
    die "genesis-init failed for environment '${TARGET_ENV}'." 7
  fi

  patch_genesis_with_generated_accounts "${TARGET_ENV}"

  if ! run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" node-bootstrap \
      --home "${AOXC_HOME_DIR}"; then
    die "node-bootstrap failed for environment '${TARGET_ENV}'." 7
  fi

  touch "${AOXC_HOME_DIR}/.bootstrap_done"
  log_info "Bootstrap completed for environment '${TARGET_ENV}'."
}

run_once() {
  local bin_path="$1"
  local tx_payload="AOXC_${TARGET_ENV}_$(date +%s)"

  export AOXC_HOME="${AOXC_HOME_DIR}"

  run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" produce-once --tx "${tx_payload}"

  run_and_tee "${RUNTIME_LOG}" \
    "${bin_path}" network-smoke \
      --timeout-ms "${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}" \
      --bind-host 127.0.0.1 \
      --port 0 \
      --payload "HEALTH_${TARGET_ENV}"
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
      log_info "Environment '${TARGET_ENV}' is already running with PID ${daemon_pid}."
      return 0
    fi

    rm -f "${PID_FILE}"
  fi

  bootstrap_env "${bin_path}"

  (
    export AOXC_HOME="${AOXC_HOME_DIR}"

    while true; do
      "${bin_path}" produce-once --tx "AOXC_${TARGET_ENV}_DAEMON_$(date +%s)" >> "${RUNTIME_LOG}" 2>&1
      "${bin_path}" network-smoke \
        --timeout-ms "${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}" \
        --bind-host 127.0.0.1 \
        --port 0 \
        --payload "HEALTH_${TARGET_ENV}" >> "${RUNTIME_LOG}" 2>&1
      sleep "${DAEMON_SLEEP_SECS:-$DEFAULT_DAEMON_SLEEP_SECS}"
    done
  ) &

  daemon_pid="$!"
  printf '%s\n' "${daemon_pid}" > "${PID_FILE}"

  if ! is_managed_pid_running "${daemon_pid}"; then
    rm -f "${PID_FILE}"
    die "Managed daemon failed to start for environment '${TARGET_ENV}'." 8
  fi

  log_info "Started environment '${TARGET_ENV}' with PID ${daemon_pid}. Log: ${RUNTIME_LOG}"
}

status_daemon() {
  local daemon_pid=''

  if [[ -f "${PID_FILE}" ]]; then
    daemon_pid="$(cat "${PID_FILE}")"
    if [[ "${daemon_pid}" =~ ^[0-9]+$ ]] && is_managed_pid_running "${daemon_pid}"; then
      log_info "Environment '${TARGET_ENV}' is running with PID ${daemon_pid}."
      return 0
    fi
  fi

  log_info "Environment '${TARGET_ENV}' is stopped."
}

stop_daemon() {
  local daemon_pid=''

  if [[ ! -f "${PID_FILE}" ]]; then
    log_info "No PID file exists for environment '${TARGET_ENV}'."
    return 0
  fi

  daemon_pid="$(cat "${PID_FILE}")"

  if [[ ! "${daemon_pid}" =~ ^[0-9]+$ ]]; then
    rm -f "${PID_FILE}"
    die "PID file contains an invalid process identifier for environment '${TARGET_ENV}'." 9
  fi

  if ! is_managed_pid_running "${daemon_pid}"; then
    rm -f "${PID_FILE}"
    log_info "Managed process is no longer running for environment '${TARGET_ENV}'. Stale PID file removed."
    return 0
  fi

  kill "${daemon_pid}" >/dev/null 2>&1 || die "Failed to send termination signal to PID ${daemon_pid}." 9

  local wait_round=0
  while is_managed_pid_running "${daemon_pid}"; do
    wait_round=$((wait_round + 1))
    if (( wait_round >= 5 )); then
      die "Managed process did not terminate within the expected interval for environment '${TARGET_ENV}'." 9
    fi
    sleep 1
  done

  rm -f "${PID_FILE}"
  log_info "Stopped environment '${TARGET_ENV}' with PID ${daemon_pid}."
}

tail_logs() {
  touch "${RUNTIME_LOG}"
  exec tail -n 100 -f "${RUNTIME_LOG}"
}

main() {
  local bin_path=''

  require_command python3
  require_command sha256sum
  require_command awk
  require_command tail

  validate_configuration
  configure_environment_profile
  initialize_runtime_paths

  if ! bin_path="$(resolve_bin_path)"; then
    die "AOXC binary not found. Run: make package-bin" 4
  fi

  case "${COMMAND}" in
    start)
      start_daemon "${bin_path}"
      ;;
    once)
      bootstrap_env "${bin_path}"
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
