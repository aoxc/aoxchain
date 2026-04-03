#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

# ==============================================================================
# AOXC Runtime Daemon
# ------------------------------------------------------------------------------
# Purpose:
#   Production-grade runtime daemon manager for AOXC.
#
# Operational goals:
#   - Deterministic runtime bootstrapping
#   - Strong PID and process supervision semantics
#   - Explicit operator receipts
#   - Premium AOXC-specific cycle rendering
#   - Strict separation between raw command output and operator-facing logs
#   - Safe failure behavior with actionable diagnostics
#
# Commands:
#   start            Start daemon in background mode
#   foreground       Run daemon loop in foreground
#   once             Execute one production/smoke cycle
#   stop             Stop daemon
#   restart          Restart daemon
#   status           Print current daemon status
#   tail             Tail premium runtime log
#   tail-raw         Tail raw runtime log
#   paths            Show resolved runtime paths
# ==============================================================================

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly COMMAND="${1:-}"

AOXC_ROOT="${AOXC_ROOT:-${HOME}/.aoxc}"

# ------------------------------------------------------------------------------
# Runtime root resolution policy
# ------------------------------------------------------------------------------
# The Makefile passes AOXC_HOME as the canonical runtime root. The daemon must
# honor that contract in order to remain consistent with the repository's
# operator surfaces.
# ------------------------------------------------------------------------------
if [[ -n "${AOXC_HOME:-}" ]]; then
  AOXC_RUNTIME_ROOT="${AOXC_RUNTIME_ROOT:-${AOXC_HOME}}"
else
  AOXC_RUNTIME_ROOT="${AOXC_RUNTIME_ROOT:-${AOXC_ROOT}/runtime}"
fi

AOXC_LOG_DIR="${AOXC_LOG_DIR:-${AOXC_ROOT}/logs}"
AOXC_AUDIT_DIR="${AOXC_AUDIT_DIR:-${AOXC_ROOT}/audit}"
AOXC_NETWORK_KIND="${AOXC_NETWORK_KIND:-mainnet}"
AOXC_RUNTIME_SOURCE_ROOT="${AOXC_RUNTIME_SOURCE_ROOT:-${ROOT_DIR}/configs/environments/${AOXC_NETWORK_KIND}}"
AOXC_BIN_OVERRIDE="${BIN_PATH:-}"

PID_FILE="${AOXC_LOG_DIR}/runtime.pid"
RUNTIME_LOG="${AOXC_LOG_DIR}/runtime.log"
RAW_RUNTIME_LOG="${AOXC_LOG_DIR}/runtime.raw.log"
STATUS_RECEIPT="${AOXC_AUDIT_DIR}/runtime-status.latest.txt"
HEALTH_RECEIPT="${AOXC_AUDIT_DIR}/runtime-health.latest.txt"
BOOTSTRAP_MARKER="${AOXC_RUNTIME_ROOT}/.bootstrap_done"

readonly DEFAULT_DAEMON_SLEEP_SECS=2
readonly DEFAULT_DAEMON_FAILURE_BACKOFF_SECS=3
readonly DEFAULT_NETWORK_TIMEOUT_MS=3000
readonly DEFAULT_START_STABILIZE_SECS=2
readonly DEFAULT_STOP_WAIT_SECS=10
readonly AOXC_CONSOLE_WIDTH=94

# ------------------------------------------------------------------------------
# Generic helpers
# ------------------------------------------------------------------------------

timestamp_utc() {
  TZ=UTC date '+%Y-%m-%dT%H:%M:%SZ'
}

repeat_char() {
  local char="$1"
  local count="$2"
  local out=""

  while (( count > 0 )); do
    out="${out}${char}"
    count=$((count - 1))
  done

  printf '%s' "${out}"
}

trim_value() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "${value}"
}

safe_value() {
  local value="${1:-}"
  [[ -n "${value}" ]] && printf '%s' "${value}" || printf '%s' "-"
}

center_text() {
  local width="$1"
  local text="$2"
  local text_len="${#text}"

  if (( text_len >= width )); then
    printf '%s' "${text}"
    return 0
  fi

  local left_pad=$(( (width - text_len) / 2 ))
  local right_pad=$(( width - text_len - left_pad ))

  printf '%s%s%s' \
    "$(repeat_char " " "${left_pad}")" \
    "${text}" \
    "$(repeat_char " " "${right_pad}")"
}

require_command() {
  local cmd="$1"
  command -v "${cmd}" >/dev/null 2>&1 || {
    printf 'ERROR: required command not found: %s\n' "${cmd}" >&2
    exit 6
  }
}

ensure_dir() {
  local dir="$1"

  if [[ -e "${dir}" && ! -d "${dir}" ]]; then
    printf 'ERROR: path exists but is not a directory: %s\n' "${dir}" >&2
    exit 6
  fi

  mkdir -p "${dir}"
}

validate_non_negative_integer() {
  local value="$1"
  local name="$2"

  [[ "${value}" =~ ^[0-9]+$ ]] || {
    printf 'ERROR: %s must be a non-negative integer. got=%s\n' "${name}" "${value}" >&2
    exit 6
  }
}

validate_positive_integer() {
  local value="$1"
  local name="$2"

  [[ "${value}" =~ ^[1-9][0-9]*$ ]] || {
    printf 'ERROR: %s must be a positive integer. got=%s\n' "${name}" "${value}" >&2
    exit 6
  }
}

append_premium_log() {
  printf '%s\n' "$*" >> "${RUNTIME_LOG}"
}

append_raw_log() {
  printf '%s\n' "$*" >> "${RAW_RUNTIME_LOG}"
}

log_info() {
  append_premium_log "$(printf '%s  INFO   %s' "$(timestamp_utc)" "$*")"
}

log_warn() {
  append_premium_log "$(printf '%s  WARN   %s' "$(timestamp_utc)" "$*")"
}

log_error() {
  append_premium_log "$(printf '%s  ERROR  %s' "$(timestamp_utc)" "$*")"
}

die() {
  local message="$1"
  local code="${2:-1}"
  log_error "${message}"
  exit "${code}"
}

initialize_paths() {
  ensure_dir "${AOXC_ROOT}"
  ensure_dir "${AOXC_RUNTIME_ROOT}"
  ensure_dir "${AOXC_LOG_DIR}"
  ensure_dir "${AOXC_AUDIT_DIR}"
  touch "${RUNTIME_LOG}"
  touch "${RAW_RUNTIME_LOG}"
}

write_status_receipt() {
  local state="$1"
  local pid_value="$2"

  {
    printf 'state=%s\n' "${state}"
    printf 'pid=%s\n' "${pid_value}"
    printf 'aoxc_root=%s\n' "${AOXC_ROOT}"
    printf 'runtime_root=%s\n' "${AOXC_RUNTIME_ROOT}"
    printf 'log_file=%s\n' "${RUNTIME_LOG}"
    printf 'raw_log_file=%s\n' "${RAW_RUNTIME_LOG}"
    printf 'timestamp_utc=%s\n' "$(timestamp_utc)"
  } > "${STATUS_RECEIPT}"
}

write_health_receipt() {
  {
    printf 'runtime_root=%s\n' "${AOXC_RUNTIME_ROOT}"
    printf 'runtime_log=%s\n' "${RUNTIME_LOG}"
    printf 'raw_runtime_log=%s\n' "${RAW_RUNTIME_LOG}"
    printf 'pid_file=%s\n' "${PID_FILE}"
    printf 'bootstrap_marker_present=%s\n' "$([[ -f "${BOOTSTRAP_MARKER}" ]] && echo yes || echo no)"
    printf 'timestamp_utc=%s\n' "$(timestamp_utc)"
  } > "${HEALTH_RECEIPT}"
}

is_pid_running() {
  local pid="$1"
  kill -0 "${pid}" >/dev/null 2>&1
}

resolve_bin_path() {
  if [[ -n "${AOXC_BIN_OVERRIDE}" ]]; then
    [[ -x "${AOXC_BIN_OVERRIDE}" ]] || die "BIN_PATH is set but not executable: ${AOXC_BIN_OVERRIDE}" 4
    printf '%s\n' "${AOXC_BIN_OVERRIDE}"
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

  return 1
}

run_and_capture() {
  local output=""
  local rc=0

  set +e
  output="$("$@" 2>&1)"
  rc=$?
  set -e

  printf '%s' "${output}"
  return "${rc}"
}

extract_yaml_like_field() {
  local payload="$1"
  local field="$2"
  local value=""

  value="$(printf '%s\n' "${payload}" \
    | awk -F': ' -v key="${field}" '$1 ~ "^[[:space:]]*" key "$" {print $2}' \
    | tail -n 1)"

  trim_value "${value}"
}

extract_or_default() {
  local payload="$1"
  local field="$2"
  local value=""

  value="$(extract_yaml_like_field "${payload}" "${field}")"
  safe_value "${value}"
}

status_icon() {
  local value="$1"

  case "${value}" in
    ok|healthy|active|true|running)
      printf '%s' "✅"
      ;;
    degraded|warning|retry)
      printf '%s' "⚠️"
      ;;
    false|stopped|failed|error)
      printf '%s' "❌"
      ;;
    *)
      printf '%s' "ℹ️"
      ;;
  esac
}

# ------------------------------------------------------------------------------
# Premium AOXC formatter
# ------------------------------------------------------------------------------

print_box_line() {
  local left="$1"
  local fill="$2"
  local right="$3"

  printf '%s%s%s\n' "${left}" "$(repeat_char "${fill}" $((AOXC_CONSOLE_WIDTH - 2)))" "${right}"
}

print_box_text_line() {
  local text="$1"
  local inner_width=$((AOXC_CONSOLE_WIDTH - 4))

  printf '║ %s ║\n' "$(center_text "${inner_width}" "${text}")"
}

print_box_kv_line() {
  local icon="$1"
  local key="$2"
  local value="$3"
  local inner_width=$((AOXC_CONSOLE_WIDTH - 4))
  local line

  line="$(printf '%-2s %-17s : %s' "${icon}" "${key}" "${value}")"
  if (( ${#line} > inner_width )); then
    line="${line:0:inner_width}"
  fi

  printf '║ %-*s ║\n' "${inner_width}" "${line}"
}

print_section_divider() {
  local icon="$1"
  local title="$2"
  local core=" ${icon} ${title} "
  local core_len="${#core}"
  local remaining=$((AOXC_CONSOLE_WIDTH - core_len))
  local left=$((remaining / 2))
  local right=$((remaining - left))

  printf '%s%s%s\n' \
    "$(repeat_char "━" "${left}")" \
    "${core}" \
    "$(repeat_char "━" "${right}")"
}

print_subtle_divider() {
  printf '%s\n' "$(repeat_char "┈" "${AOXC_CONSOLE_WIDTH}")"
}

print_kv() {
  local icon="$1"
  local key="$2"
  local value="$3"

  printf '  %-2s %-29s : %s\n' "${icon}" "${key}" "${value}"
}

print_cycle_header() {
  local tx_id="$1"
  local blk="$2"
  local height="$3"
  local round="$4"
  local smoke="$5"
  local probe="$6"
  local state_icon

  state_icon="$(status_icon "${smoke}")"

  print_box_line "╔" "═" "╗"
  print_box_text_line "🚀 AOXC RUNTIME NODE CYCLE"
  print_box_line "╠" "═" "╣"
  print_box_kv_line "${state_icon}" "Status" "HEALTHY"
  print_box_kv_line "🧾" "Transaction" "${tx_id}"
  print_box_kv_line "🧱" "Block" "${blk}"
  print_box_kv_line "📏" "Height" "${height}"
  print_box_kv_line "🔁" "Round" "${round}"
  print_box_kv_line "🧪" "Smoke Check" "${smoke}"
  print_box_kv_line "📡" "Probe" "${probe}"
  print_box_line "╚" "═" "╝"
}

print_consensus_block() {
  local produce_payload="$1"

  print_section_divider "⚖️" "CONSENSUS SNAPSHOT"
  print_kv "🌐" "Network ID"             "$(extract_or_default "${produce_payload}" "network_id")"
  print_kv "📏" "Current Height"         "$(extract_or_default "${produce_payload}" "current_height")"
  print_kv "🔁" "Last Round"             "$(extract_or_default "${produce_payload}" "last_round")"
  print_kv "🧩" "Last Message Kind"      "$(extract_or_default "${produce_payload}" "last_message_kind")"
  print_kv "📚" "Last Section Count"     "$(extract_or_default "${produce_payload}" "last_section_count")"
  print_kv "⏱️" "Last Timestamp (Unix)"   "$(extract_or_default "${produce_payload}" "last_timestamp_unix")"
  print_subtle_divider
  print_kv "🔐" "Last Block Hash"        "$(extract_or_default "${produce_payload}" "last_block_hash_hex")"
  print_kv "🧬" "Parent Block Hash"      "$(extract_or_default "${produce_payload}" "last_parent_hash_hex")"
  print_kv "👤" "Proposer Public Key"    "$(extract_or_default "${produce_payload}" "last_proposer_hex")"
}

print_key_material_block() {
  local produce_payload="$1"

  print_section_divider "🔑" "KEY MATERIAL STATUS"
  print_kv "🪪" "Bundle Fingerprint"     "$(extract_or_default "${produce_payload}" "bundle_fingerprint")"
  print_kv "🛡️" "Consensus Key"          "$(extract_or_default "${produce_payload}" "consensus_public_key_hex")"
  print_kv "🔄" "Transport Key"          "$(extract_or_default "${produce_payload}" "transport_public_key_hex")"
  print_kv "$(status_icon "$(extract_or_default "${produce_payload}" "operational_state")")" \
           "Operational State"          "$(extract_or_default "${produce_payload}" "operational_state")"
}

print_runtime_block() {
  local produce_payload="$1"

  print_section_divider "⚙️" "RUNTIME EXECUTION"
  print_kv "$(status_icon "$(extract_or_default "${produce_payload}" "initialized")")" \
           "Initialized"                "$(extract_or_default "${produce_payload}" "initialized")"
  print_kv "$(status_icon "$(extract_or_default "${produce_payload}" "running")")" \
           "Running"                    "$(extract_or_default "${produce_payload}" "running")"
  print_kv "📦" "Produced Blocks"       "$(extract_or_default "${produce_payload}" "produced_blocks")"
  print_kv "📏" "Current Height"        "$(extract_or_default "${produce_payload}" "current_height")"
  print_kv "🧾" "Last Transaction"      "$(extract_or_default "${produce_payload}" "last_tx")"
  print_kv "🕒" "Updated At"            "$(extract_or_default "${produce_payload}" "updated_at")"
}

print_network_block() {
  local smoke_payload="$1"

  print_section_divider "📡" "NETWORK SMOKE TEST"
  print_kv "🛠️" "Command"               "$(extract_or_default "${smoke_payload}" "command")"
  print_kv "$(status_icon "$(extract_or_default "${smoke_payload}" "status")")" \
           "Status"                     "$(extract_or_default "${smoke_payload}" "status")"
  print_kv "🌍" "Bind Host"             "$(extract_or_default "${smoke_payload}" "bind_host")"
  print_kv "🚪" "RPC Port"              "$(extract_or_default "${smoke_payload}" "rpc_port")"
  print_kv "📶" "Probe Result"          "$(extract_or_default "${smoke_payload}" "probe")"
  print_kv "$(status_icon "$(extract_or_default "${smoke_payload}" "key_operational_state")")" \
           "Key Operational State"      "$(extract_or_default "${smoke_payload}" "key_operational_state")"
  print_kv "🔄" "Transport Public Key"  "$(extract_or_default "${smoke_payload}" "transport_public_key")"
  print_kv "🕒" "Timestamp"             "$(extract_or_default "${smoke_payload}" "timestamp")"
}

print_identity_footer() {
  print_section_divider "🏛️" "AOXC NODE MARK"
  print_kv "🧬" "Identity Class"        "AOXC Validator Runtime"
  print_kv "🖥️" "Runtime Mode"          "Daemon Cycle"
  print_kv "🎛️" "Output Profile"        "Professional Operator Console"
  print_kv "🔗" "Chain Signature"       "AOXC Premium Structured Telemetry"
}

render_aoxc_cycle_console() {
  local produce_payload="$1"
  local smoke_payload="$2"

  local tx_id blk height round smoke probe

  tx_id="$(extract_or_default "${produce_payload}" "last_tx")"
  blk="$(extract_or_default "${produce_payload}" "produced_blocks")"
  height="$(extract_or_default "${produce_payload}" "current_height")"
  round="$(extract_or_default "${produce_payload}" "last_round")"
  smoke="$(extract_or_default "${smoke_payload}" "status")"
  probe="$(extract_or_default "${smoke_payload}" "probe")"

  print_cycle_header "${tx_id}" "${blk}" "${height}" "${round}" "${smoke}" "${probe}"
  printf '\n'
  print_consensus_block "${produce_payload}"
  printf '\n'
  print_key_material_block "${produce_payload}"
  printf '\n'
  print_runtime_block "${produce_payload}"
  printf '\n'
  print_network_block "${smoke_payload}"
  printf '\n'
  print_identity_footer
}

emit_professional_cycle_log() {
  local produce_payload="$1"
  local smoke_payload="$2"

  {
    printf '\n'
    render_aoxc_cycle_console "${produce_payload}" "${smoke_payload}"
    printf '\n'
  } >> "${RUNTIME_LOG}"
}

emit_cycle_failure_log() {
  local tx_id="$1"
  local stage="$2"
  local exit_code="$3"

  {
    printf '\n'
    print_section_divider "⚠️" "AOXC RUNTIME CYCLE FAILURE"
    print_kv "🧾" "Transaction" "${tx_id}"
    print_kv "🛠️" "Failed Stage" "${stage}"
    print_kv "❌" "Exit Code" "${exit_code}"
    print_kv "📄" "Raw Log" "${RAW_RUNTIME_LOG}"
    printf '\n'
  } >> "${RUNTIME_LOG}"
}

# ------------------------------------------------------------------------------
# Runtime installation/bootstrap
# ------------------------------------------------------------------------------

copy_runtime_source_if_present() {
  if [[ ! -d "${AOXC_RUNTIME_SOURCE_ROOT}" ]]; then
    log_warn "Runtime source root not found: ${AOXC_RUNTIME_SOURCE_ROOT}"
    return 0
  fi

  mkdir -p "${AOXC_RUNTIME_ROOT}/identity" "${AOXC_RUNTIME_ROOT}/config"

  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/manifest.v1.json"    ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/manifest.v1.json"    "${AOXC_RUNTIME_ROOT}/identity/manifest.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/genesis.v1.json"     ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/genesis.v1.json"     "${AOXC_RUNTIME_ROOT}/identity/genesis.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/validators.json"     ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/validators.json"     "${AOXC_RUNTIME_ROOT}/identity/validators.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/bootnodes.json"      ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/bootnodes.json"      "${AOXC_RUNTIME_ROOT}/identity/bootnodes.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/certificate.json"    ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/certificate.json"    "${AOXC_RUNTIME_ROOT}/identity/certificate.json"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/profile.toml"        ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/profile.toml"        "${AOXC_RUNTIME_ROOT}/config/profile.toml"
  [[ -f "${AOXC_RUNTIME_SOURCE_ROOT}/release-policy.toml" ]] && cp "${AOXC_RUNTIME_SOURCE_ROOT}/release-policy.toml" "${AOXC_RUNTIME_ROOT}/config/release-policy.toml"

  if [[ -f "${AOXC_RUNTIME_ROOT}/identity/genesis.json" ]]; then
    sha256sum "${AOXC_RUNTIME_ROOT}/identity/genesis.json" > "${AOXC_RUNTIME_ROOT}/identity/genesis.sha256"
  fi

  log_info "Runtime source material installed into ${AOXC_RUNTIME_ROOT}"
}

bootstrap_runtime() {
  local bin_path="$1"
  export AOXC_HOME="${AOXC_RUNTIME_ROOT}"

  if [[ -f "${BOOTSTRAP_MARKER}" ]]; then
    log_info "Bootstrap marker already present. Reusing initialized runtime root."
    return 0
  fi

  log_info "Starting runtime bootstrap"
  log_info "Resolved binary: ${bin_path}"
  log_info "Runtime root: ${AOXC_RUNTIME_ROOT}"
  log_info "Source root: ${AOXC_RUNTIME_SOURCE_ROOT}"

  copy_runtime_source_if_present

  local db_init_output=""
  db_init_output="$(run_and_capture "${bin_path}" db-init --backend redb --format json)" || {
    append_raw_log "${db_init_output}"
    die "db-init failed during bootstrap" 7
  }

  append_raw_log "${db_init_output}"
  touch "${BOOTSTRAP_MARKER}"
  log_info "Runtime bootstrap completed successfully"
}

# ------------------------------------------------------------------------------
# Runtime cycles
# ------------------------------------------------------------------------------

run_cycle() {
  local bin_path="$1"
  local mode="$2"
  local timeout_ms="${NETWORK_TIMEOUT_MS:-$DEFAULT_NETWORK_TIMEOUT_MS}"

  validate_positive_integer "${timeout_ms}" "NETWORK_TIMEOUT_MS"
  export AOXC_HOME="${AOXC_RUNTIME_ROOT}"

  local tx_id="AOXC_RUNTIME_${mode^^}_$(date +%s)"
  local produce_output=""
  local smoke_output=""
  local produce_rc=0
  local smoke_rc=0

  set +e
  produce_output="$("${bin_path}" produce-once --tx "${tx_id}" 2>&1)"
  produce_rc=$?
  set -e

  append_raw_log "${produce_output}"

  if (( produce_rc != 0 )); then
    emit_cycle_failure_log "${tx_id}" "produce-once" "${produce_rc}"
    return 1
  fi

  set +e
  smoke_output="$("${bin_path}" network-smoke \
    --timeout-ms "${timeout_ms}" \
    --bind-host 127.0.0.1 \
    --port 0 \
    --payload "HEALTH_RUNTIME" 2>&1)"
  smoke_rc=$?
  set -e

  append_raw_log "${smoke_output}"

  if (( smoke_rc != 0 )); then
    emit_cycle_failure_log "${tx_id}" "network-smoke" "${smoke_rc}"
    return 1
  fi

  emit_professional_cycle_log "${produce_output}" "${smoke_output}"
  write_health_receipt
  return 0
}

run_daemon_loop() {
  local bin_path="$1"
  local mode="$2"

  local sleep_secs="${DAEMON_SLEEP_SECS:-$DEFAULT_DAEMON_SLEEP_SECS}"
  local backoff_secs="${DAEMON_FAILURE_BACKOFF_SECS:-$DEFAULT_DAEMON_FAILURE_BACKOFF_SECS}"

  validate_non_negative_integer "${sleep_secs}" "DAEMON_SLEEP_SECS"
  validate_non_negative_integer "${backoff_secs}" "DAEMON_FAILURE_BACKOFF_SECS"

  log_info "Entering daemon loop mode=${mode}"

  while true; do
    if ! run_cycle "${bin_path}" "${mode}"; then
      log_warn "Cycle failed in mode=${mode}; retrying in ${backoff_secs}s"
      sleep "${backoff_secs}"
      continue
    fi

    sleep "${sleep_secs}"
  done
}

# ------------------------------------------------------------------------------
# Commands
# ------------------------------------------------------------------------------

start_daemon() {
  local bin_path="$1"
  local stabilize_secs="${START_STABILIZE_SECS:-$DEFAULT_START_STABILIZE_SECS}"
  local pid=""

  validate_non_negative_integer "${stabilize_secs}" "START_STABILIZE_SECS"

  if [[ -f "${PID_FILE}" ]]; then
    pid="$(cat "${PID_FILE}")"
    if [[ "${pid}" =~ ^[0-9]+$ ]] && is_pid_running "${pid}"; then
      write_status_receipt "running" "${pid}"
      write_health_receipt
      printf 'AOXC runtime already running. pid=%s\n' "${pid}"
      return 0
    fi
    rm -f "${PID_FILE}"
    log_warn "Removed stale PID file"
  fi

  bootstrap_runtime "${bin_path}"

  (
    exec env \
      AOXC_ROOT="${AOXC_ROOT}" \
      AOXC_HOME="${AOXC_RUNTIME_ROOT}" \
      AOXC_RUNTIME_ROOT="${AOXC_RUNTIME_ROOT}" \
      AOXC_LOG_DIR="${AOXC_LOG_DIR}" \
      AOXC_AUDIT_DIR="${AOXC_AUDIT_DIR}" \
      AOXC_NETWORK_KIND="${AOXC_NETWORK_KIND}" \
      AOXC_RUNTIME_SOURCE_ROOT="${AOXC_RUNTIME_SOURCE_ROOT}" \
      BIN_PATH="${bin_path}" \
      "${SCRIPT_DIR}/runtime_daemon.sh" foreground
  ) >/dev/null 2>&1 &

  pid="$!"
  printf '%s\n' "${pid}" > "${PID_FILE}"

  sleep "${stabilize_secs}"

  if ! is_pid_running "${pid}"; then
    rm -f "${PID_FILE}"
    write_status_receipt "failed" "none"
    die "Runtime daemon exited during stabilization window. Inspect ${RUNTIME_LOG} and ${RAW_RUNTIME_LOG}" 8
  fi

  write_status_receipt "running" "${pid}"
  write_health_receipt

  printf 'AOXC runtime started successfully.\n'
  printf '  PID File   : %s\n' "${PID_FILE}"
  printf '  PID        : %s\n' "${pid}"
  printf '  Log File   : %s\n' "${RUNTIME_LOG}"
  printf '  Raw Log    : %s\n' "${RAW_RUNTIME_LOG}"
  printf '  Status     : %s\n' "${STATUS_RECEIPT}"
}

run_foreground() {
  local bin_path="$1"

  bootstrap_runtime "${bin_path}"
  write_status_receipt "running" "$$"
  write_health_receipt
  run_daemon_loop "${bin_path}" "daemon"
}

run_once() {
  local bin_path="$1"

  bootstrap_runtime "${bin_path}"

  if run_cycle "${bin_path}" "once"; then
    write_status_receipt "single-run-complete" "$$"
    printf 'AOXC single runtime cycle completed successfully.\n'
    return 0
  fi

  write_status_receipt "single-run-failed" "$$"
  die "Single runtime cycle failed. Inspect ${RUNTIME_LOG} and ${RAW_RUNTIME_LOG}" 8
}

stop_daemon() {
  local max_wait="${STOP_WAIT_SECS:-$DEFAULT_STOP_WAIT_SECS}"
  local pid=""
  local waited=0

  validate_non_negative_integer "${max_wait}" "STOP_WAIT_SECS"

  if [[ ! -f "${PID_FILE}" ]]; then
    write_status_receipt "stopped" "none"
    printf 'AOXC runtime is already stopped.\n'
    return 0
  fi

  pid="$(cat "${PID_FILE}")"

  if [[ ! "${pid}" =~ ^[0-9]+$ ]]; then
    rm -f "${PID_FILE}"
    die "PID file content is invalid" 9
  fi

  if ! is_pid_running "${pid}"; then
    rm -f "${PID_FILE}"
    write_status_receipt "stopped" "none"
    printf 'Removed stale AOXC runtime PID file.\n'
    return 0
  fi

  kill "${pid}" >/dev/null 2>&1 || die "Unable to signal PID ${pid}" 9

  while is_pid_running "${pid}"; do
    sleep 1
    waited=$((waited + 1))
    if (( waited >= max_wait )); then
      die "Runtime did not stop within ${max_wait}s" 9
    fi
  done

  rm -f "${PID_FILE}"
  write_status_receipt "stopped" "none"
  write_health_receipt

  printf 'AOXC runtime stopped successfully. pid=%s\n' "${pid}"
}

status_daemon() {
  local pid=""

  printf 'AOXC Runtime Status\n'
  printf '%s\n' "$(repeat_char "─" 48)"
  printf 'AOXC Root      : %s\n' "${AOXC_ROOT}"
  printf 'Runtime Root   : %s\n' "${AOXC_RUNTIME_ROOT}"
  printf 'Log File       : %s\n' "${RUNTIME_LOG}"
  printf 'Raw Log File   : %s\n' "${RAW_RUNTIME_LOG}"
  printf 'PID File       : %s\n' "${PID_FILE}"
  printf 'Status Receipt : %s\n' "${STATUS_RECEIPT}"
  printf 'Health Receipt : %s\n' "${HEALTH_RECEIPT}"
  printf '\n'

  if [[ -f "${PID_FILE}" ]]; then
    pid="$(cat "${PID_FILE}")"
    if [[ "${pid}" =~ ^[0-9]+$ ]] && is_pid_running "${pid}"; then
      write_status_receipt "running" "${pid}"
      write_health_receipt
      printf 'State          : RUNNING ✅\n'
      printf 'PID            : %s\n' "${pid}"
      return 0
    fi
  fi

  write_status_receipt "stopped" "none"
  write_health_receipt
  printf 'State          : STOPPED ❌\n'
}

tail_logs() {
  touch "${RUNTIME_LOG}"
  printf 'Tailing AOXC premium runtime log: %s\n' "${RUNTIME_LOG}"
  exec tail -n 200 -f "${RUNTIME_LOG}"
}

tail_raw_logs() {
  touch "${RAW_RUNTIME_LOG}"
  printf 'Tailing AOXC raw runtime log: %s\n' "${RAW_RUNTIME_LOG}"
  exec tail -n 200 -f "${RAW_RUNTIME_LOG}"
}

print_paths() {
  local bin_path="-"
  bin_path="$(resolve_bin_path || true)"

  printf 'AOXC Runtime Paths\n'
  printf '%s\n' "$(repeat_char "─" 48)"
  printf 'ROOT_DIR                 : %s\n' "${ROOT_DIR}"
  printf 'SCRIPT_DIR               : %s\n' "${SCRIPT_DIR}"
  printf 'AOXC_ROOT                : %s\n' "${AOXC_ROOT}"
  printf 'AOXC_RUNTIME_ROOT        : %s\n' "${AOXC_RUNTIME_ROOT}"
  printf 'AOXC_LOG_DIR             : %s\n' "${AOXC_LOG_DIR}"
  printf 'AOXC_AUDIT_DIR           : %s\n' "${AOXC_AUDIT_DIR}"
  printf 'AOXC_RUNTIME_SOURCE_ROOT : %s\n' "${AOXC_RUNTIME_SOURCE_ROOT}"
  printf 'RESOLVED_BIN             : %s\n' "${bin_path}"
  printf 'PID_FILE                 : %s\n' "${PID_FILE}"
  printf 'RUNTIME_LOG              : %s\n' "${RUNTIME_LOG}"
  printf 'RAW_RUNTIME_LOG          : %s\n' "${RAW_RUNTIME_LOG}"
  printf 'STATUS_RECEIPT           : %s\n' "${STATUS_RECEIPT}"
  printf 'HEALTH_RECEIPT           : %s\n' "${HEALTH_RECEIPT}"
}

usage() {
  cat <<'EOF'
Usage:
  ./scripts/runtime_daemon.sh <start|foreground|once|stop|restart|status|tail|tail-raw|paths>
EOF
}

main() {
  local bin_path=""

  require_command awk
  require_command tail
  require_command sha256sum

  initialize_paths

  [[ -n "${COMMAND}" ]] || {
    usage
    exit 2
  }

  case "${COMMAND}" in
    start)
      bin_path="$(resolve_bin_path)" || die "AOXC binary not found. Build/install it first." 4
      start_daemon "${bin_path}"
      ;;
    foreground)
      bin_path="$(resolve_bin_path)" || die "AOXC binary not found. Build/install it first." 4
      run_foreground "${bin_path}"
      ;;
    once)
      bin_path="$(resolve_bin_path)" || die "AOXC binary not found. Build/install it first." 4
      run_once "${bin_path}"
      ;;
    stop)
      stop_daemon
      ;;
    restart)
      stop_daemon || true
      bin_path="$(resolve_bin_path)" || die "AOXC binary not found. Build/install it first." 4
      start_daemon "${bin_path}"
      ;;
    status)
      status_daemon
      ;;
    tail)
      tail_logs
      ;;
    tail-raw)
      tail_raw_logs
      ;;
    paths)
      print_paths
      ;;
    -h|--help|help)
      usage
      ;;
    *)
      die "Unknown command: ${COMMAND}" 5
      ;;
  esac
}

main "$@"
