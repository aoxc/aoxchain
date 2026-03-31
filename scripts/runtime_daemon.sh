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
#   - Support `start`, `once`, `status`, `stop`, `restart`, `tail`,
#     `run-foreground`, and `install-service`
#   - Persist PID-based lifecycle state for managed background execution
#   - Emit deterministic status and health receipts
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
#   10 Service installation failure
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

readonly DEFAULT_NETWORK_TIMEOUT_MS=3000
readonly DEFAULT_DAEMON_SLEEP_SECS=2
readonly DEFAULT_STOP_WAIT_SECS=10
readonly DEFAULT_DAEMON_FAILURE_BACKOFF_SECS=3

readonly COMMAND="${1:-}"

AOXC_ROOT="${AOXC_ROOT:-${HOME}/.aoxc}"
AOXC_RUNTIME_ROOT="${AOXC_RUNTIME_ROOT:-${AOXC_ROOT}/runtime}"
AOXC_LOG_DIR="${AOXC_LOG_DIR:-${AOXC_ROOT}/logs}"
AOXC_BIN_PATH_OVERRIDE="${BIN_PATH:-}"
AOXC_NETWORK_KIND="${AOXC_NETWORK_KIND:-mainnet}"
AOXC_RUNTIME_SOURCE_ROOT="${AOXC_RUNTIME_SOURCE_ROOT:-${ROOT_DIR}/configs/environments/${AOXC_NETWORK_KIND}}"

PID_FILE="${AOXC_LOG_DIR}/runtime.pid"
RUNTIME_LOG="${AOXC_LOG_DIR}/runtime.log"
BOOTSTRAP_MARKER="${AOXC_RUNTIME_ROOT}/.bootstrap_done"
HEALTH_RECEIPT="${AOXC_LOG_DIR}/runtime-health.latest.txt"
STATUS_RECEIPT="${AOXC_LOG_DIR}/runtime-status.latest.txt"

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
  cat <<'USAGE'
Usage:
  ./scripts/runtime_daemon.sh <start|once|status|stop|restart|tail|run-foreground|install-service>

Environment:
  AOXC_SYSTEMD_SCOPE=user|system (default: user)
  AOXC_SYSTEMD_SERVICE_NAME=<service-name> (default: aoxc-runtime)
USAGE
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

validate_positive_integer() {
  local value="$1"
  local name="$2"

  [[ "${value}" =~ ^[1-9][0-9]*$ ]] || die "Invalid value for ${name}: '${value}'. A positive integer is required." 6
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

copy_runtime_source_if_present() {
  if [[ ! -d "${AOXC_RUNTIME_SOURCE_ROOT}" ]]; then
    log_warn "Runtime source root is absent: ${AOXC_RUNTIME_SOURCE_ROOT}. Source materialization will be skipped."
    return 0
  fi

  ensure_directory "${AOXC_RUNTIME_ROOT}/identity"
  ensure_directory "${AOXC_RUNTIME_ROOT}/config"

  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/manifest.v1.json" ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/manifest.v1.json" "${AOXC_RUNTIME_ROOT}/identity/manifest.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/genesis.v1.json" ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/genesis.v1.json" "${AOXC_RUNTIME_ROOT}/identity/genesis.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/validators.json" ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/validators.json" "${AOXC_RUNTIME_ROOT}/identity/validators.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/bootnodes.json" ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/bootnodes.json" "${AOXC_RUNTIME_ROOT}/identity/bootnodes.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/certificate.json" ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/certificate.json" "${AOXC_RUNTIME_ROOT}/identity/certificate.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/profile.toml" ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/profile.toml" "${AOXC_RUNTIME_ROOT}/config/profile.toml"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/release-policy.toml" ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/release-policy.toml" "${AOXC_RUNTIME_ROOT}/config/release-policy.toml"

  if [[ -f "${AOXC_RUNTIME_ROOT}/identity/genesis.json" ]]; then
    sha256sum "${AOXC_RUNTIME_ROOT}/identity/genesis.json" > "${AOXC_RUNTIME_ROOT}/identity/genesis.sha256"
  fi
}

bootstrap_runtime() {
  local bin_path="$1"

  export AOXC_HOME="${AOXC_RUNTIME_ROOT}"

  if [[ -f "${BOOTSTRAP_MARKER}" ]]; then
    log_info "Bootstrap already completed for runtime root '${AOXC_RUNTIME_ROOT}'."
    return 0
  fi

  log_info "Bootstrap started for runtime root '${AOXC_RUNTIME_ROOT}'."
  log_info "Resolved network kind '${AOXC_NETWORK_KIND}' with source root '${AOXC_RUNTIME_SOURCE_ROOT}'."

  copy_runtime_source_if_present

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

write_status_receipt() {
  local state="$1"
  local pid_value="$2"

  {
    printf 'state=%s\n' "${state}"
    printf 'pid=%s\n' "${pid_value}"
    printf 'runtime_root=%s\n' "${AOXC_RUNTIME_ROOT}"
    printf 'log_dir=%s\n' "${AOXC_LOG_DIR}"
    printf 'timestamp_utc=%s\n' "$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
  } > "${STATUS_RECEIPT}"
}

write_health_receipt() {
  {
    printf 'runtime_root=%s\n' "${AOXC_RUNTIME_ROOT}"
    printf 'log_dir=%s\n' "${AOXC_LOG_DIR}"
    printf 'pid_file=%s\n' "${PID_FILE}"
    printf 'runtime_log=%s\n' "${RUNTIME_LOG}"
    printf 'bootstrap_marker_present=%s\n' "$([[ -f "${BOOTSTRAP_MARKER}" ]] && echo yes || echo no)"
    printf 'timestamp_utc=%s\n' "$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
  } > "${HEALTH_RECEIPT}"
}

start_daemon() {
  local bin_path="$1"
  local daemon_pid=''

  if [[ -f "${PID_FILE}" ]]; then
    daemon_pid="$(cat "${PID_FILE}")"
    if [[ "${daemon_pid}" =~ ^[0-9]+$ ]] && is_managed_pid_running "${daemon_pid}"; then
      write_status_receipt "running" "${daemon_pid}"
      write_health_receipt
      log_info "Runtime is already running with PID ${daemon_pid}."
      return 0
    fi

    rm -f "${PID_FILE}"
  fi

  bootstrap_runtime "${bin_path}"

  (
    export AOXC_HOME="${AOXC_RUNTIME_ROOT}"
    local backoff_secs="${DAEMON_FAILURE_BACKOFF_SECS:-$DEFAULT_DAEMON_FAILURE_BACKOFF_SECS}"
    local sleep_secs="${DAEMON_SLEEP_SECS:-$DEFAULT_DAEMON_SLEEP_SECS}"
    local timeout_ms="${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}"
    local tx_suffix=''
    local produce_rc=0
    local smoke_rc=0

    validate_non_negative_integer "${backoff_secs}" "DAEMON_FAILURE_BACKOFF_SECS"

    while true; do
      tx_suffix="$(date +%s)"

      set +e
      "${bin_path}" produce-once --tx "AOXC_RUNTIME_DAEMON_${tx_suffix}" >> "${RUNTIME_LOG}" 2>&1
      produce_rc=$?
      "${bin_path}" network-smoke \
        --timeout-ms "${timeout_ms}" \
        --bind-host 127.0.0.1 \
        --port 0 \
        --payload "HEALTH_RUNTIME" >> "${RUNTIME_LOG}" 2>&1
      smoke_rc=$?
      set -e

      if (( produce_rc != 0 || smoke_rc != 0 )); then
        printf '[runtime-daemon][warn] cycle failed (produce-once=%s network-smoke=%s). Retrying in %ss.\n' "${produce_rc}" "${smoke_rc}" "${backoff_secs}" >> "${RUNTIME_LOG}"
        sleep "${backoff_secs}"
        continue
      fi

      sleep "${sleep_secs}"
    done
  ) &

  daemon_pid="$!"
  printf '%s\n' "${daemon_pid}" > "${PID_FILE}"

  if ! is_managed_pid_running "${daemon_pid}"; then
    rm -f "${PID_FILE}"
    die "Managed daemon failed to start." 8
  fi

  write_status_receipt "running" "${daemon_pid}"
  write_health_receipt
  log_info "Started runtime with PID ${daemon_pid}. Log: ${RUNTIME_LOG}"
}

run_foreground() {
  local bin_path="$1"
  local backoff_secs="${DAEMON_FAILURE_BACKOFF_SECS:-$DEFAULT_DAEMON_FAILURE_BACKOFF_SECS}"
  local sleep_secs="${DAEMON_SLEEP_SECS:-$DEFAULT_DAEMON_SLEEP_SECS}"
  local timeout_ms="${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}"
  local tx_suffix=''
  local produce_rc=0
  local smoke_rc=0

  validate_non_negative_integer "${backoff_secs}" "DAEMON_FAILURE_BACKOFF_SECS"
  validate_non_negative_integer "${sleep_secs}" "DAEMON_SLEEP_SECS"
  validate_positive_integer "${timeout_ms}" "NETWORK_TIMEOUT_MS"

  bootstrap_runtime "${bin_path}"
  export AOXC_HOME="${AOXC_RUNTIME_ROOT}"

  log_info "Running foreground runtime loop for persistent service mode."
  while true; do
    tx_suffix="$(date +%s)"

    set +e
    "${bin_path}" produce-once --tx "AOXC_RUNTIME_FOREGROUND_${tx_suffix}" >> "${RUNTIME_LOG}" 2>&1
    produce_rc=$?
    "${bin_path}" network-smoke \
      --timeout-ms "${timeout_ms}" \
      --bind-host 127.0.0.1 \
      --port 0 \
      --payload "HEALTH_RUNTIME" >> "${RUNTIME_LOG}" 2>&1
    smoke_rc=$?
    set -e

    if (( produce_rc != 0 || smoke_rc != 0 )); then
      log_warn "Foreground cycle failed (produce-once=${produce_rc}, network-smoke=${smoke_rc}); retrying in ${backoff_secs}s."
      sleep "${backoff_secs}"
      continue
    fi

    sleep "${sleep_secs}"
  done
}

status_daemon() {
  local daemon_pid=''

  if [[ -f "${PID_FILE}" ]]; then
    daemon_pid="$(cat "${PID_FILE}")"
    if [[ "${daemon_pid}" =~ ^[0-9]+$ ]] && is_managed_pid_running "${daemon_pid}"; then
      write_status_receipt "running" "${daemon_pid}"
      write_health_receipt
      log_info "Runtime is running with PID ${daemon_pid}."
      return 0
    fi
  fi

  write_status_receipt "stopped" "none"
  write_health_receipt
  log_info "Runtime is stopped."
}

stop_daemon() {
  local daemon_pid=''
  local wait_round=0
  local max_wait="${STOP_WAIT_SECS:-$DEFAULT_STOP_WAIT_SECS}"

  validate_non_negative_integer "${max_wait}" "STOP_WAIT_SECS"

  if [[ ! -f "${PID_FILE}" ]]; then
    write_status_receipt "stopped" "none"
    write_health_receipt
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
    write_status_receipt "stopped" "none"
    write_health_receipt
    log_info "Managed process is no longer running. Stale PID file removed."
    return 0
  fi

  kill "${daemon_pid}" >/dev/null 2>&1 || die "Failed to send termination signal to PID ${daemon_pid}." 9

  while is_managed_pid_running "${daemon_pid}"; do
    wait_round=$((wait_round + 1))
    if (( wait_round >= max_wait )); then
      die "Managed process did not terminate within the expected interval." 9
    fi
    sleep 1
  done

  rm -f "${PID_FILE}"
  write_status_receipt "stopped" "none"
  write_health_receipt
  log_info "Stopped runtime with PID ${daemon_pid}."
}

tail_logs() {
  touch "${RUNTIME_LOG}"
  exec tail -n 100 -f "${RUNTIME_LOG}"
}

install_service() {
  local scope="${AOXC_SYSTEMD_SCOPE:-user}"
  local service_name="${AOXC_SYSTEMD_SERVICE_NAME:-aoxc-runtime}"
  local runtime_script="${SCRIPT_DIR}/runtime_daemon.sh"
  local unit_dir=''
  local service_file=''

  [[ "${service_name}" =~ ^[A-Za-z0-9._-]+$ ]] || die "Invalid AOXC_SYSTEMD_SERVICE_NAME: '${service_name}'." 10
  [[ -x "${runtime_script}" ]] || chmod +x "${runtime_script}" || die "Unable to mark runtime script executable: ${runtime_script}" 10

  case "${scope}" in
    user)
      unit_dir="${HOME}/.config/systemd/user"
      require_command systemctl
      ;;
    system)
      unit_dir="/etc/systemd/system"
      require_command systemctl
      if [[ "${EUID}" -ne 0 ]]; then
        die "System scope requires root privileges (AOXC_SYSTEMD_SCOPE=system)." 10
      fi
      ;;
    *)
      die "Invalid AOXC_SYSTEMD_SCOPE: '${scope}'. Use 'user' or 'system'." 10
      ;;
  esac

  ensure_directory "${unit_dir}"
  service_file="${unit_dir}/${service_name}.service"

  cat > "${service_file}" <<SERVICE
[Unit]
Description=AOXC Persistent Runtime Daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=${ROOT_DIR}
Environment=AOXC_ROOT=${AOXC_ROOT}
Environment=AOXC_RUNTIME_ROOT=${AOXC_RUNTIME_ROOT}
Environment=AOXC_LOG_DIR=${AOXC_LOG_DIR}
Environment=AOXC_NETWORK_KIND=${AOXC_NETWORK_KIND}
Environment=AOXC_RUNTIME_SOURCE_ROOT=${AOXC_RUNTIME_SOURCE_ROOT}
ExecStop=${runtime_script} stop
ExecStart=${runtime_script} run-foreground
Restart=always
RestartSec=5
TimeoutStopSec=30

[Install]
WantedBy=default.target
SERVICE

  if [[ "${scope}" == "user" ]]; then
    systemctl --user daemon-reload || die "systemctl --user daemon-reload failed." 10
    systemctl --user enable "${service_name}.service" || die "Unable to enable user service '${service_name}'." 10
    log_info "Service installed: ${service_file}"
    log_info "Start with: systemctl --user start ${service_name}.service"
    log_info "Enable linger for reboot persistence: loginctl enable-linger ${USER}"
  else
    systemctl daemon-reload || die "systemctl daemon-reload failed." 10
    systemctl enable "${service_name}.service" || die "Unable to enable system service '${service_name}'." 10
    log_info "Service installed: ${service_file}"
    log_info "Start with: systemctl start ${service_name}.service"
  fi
}

main() {
  local bin_path=''

  require_command sha256sum
  require_command awk
  require_command tail

  validate_positive_integer "${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}" "NETWORK_TIMEOUT_MS"
  validate_non_negative_integer "${DAEMON_SLEEP_SECS:-$DEFAULT_DAEMON_SLEEP_SECS}" "DAEMON_SLEEP_SECS"

  [[ -n "${COMMAND}" ]] || {
    print_usage >&2
    exit 2
  }

  initialize_runtime_paths

  case "${COMMAND}" in
    start)
      if ! bin_path="$(resolve_bin_path)"; then
        die "AOXC binary not found. Run: make package-bin" 4
      fi
      start_daemon "${bin_path}"
      ;;
    once)
      if ! bin_path="$(resolve_bin_path)"; then
        die "AOXC binary not found. Run: make package-bin" 4
      fi
      bootstrap_runtime "${bin_path}"
      run_once "${bin_path}"
      write_health_receipt
      ;;
    status)
      status_daemon
      ;;
    stop)
      stop_daemon
      ;;
    restart)
      if ! bin_path="$(resolve_bin_path)"; then
        die "AOXC binary not found. Run: make package-bin" 4
      fi
      stop_daemon
      start_daemon "${bin_path}"
      ;;
    tail)
      tail_logs
      ;;
    install-service)
      install_service
      ;;
    run-foreground)
      if ! bin_path="$(resolve_bin_path)"; then
        die "AOXC binary not found. Run: make package-bin" 4
      fi
      run_foreground "${bin_path}"
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
