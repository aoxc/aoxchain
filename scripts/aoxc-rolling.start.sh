#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -Eeuo pipefail
IFS=$'\n\t'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

AOXC_Q_HOME="${AOXC_Q_HOME:-/mnt/xdbx/aoxc}"
AOXC_Q_ENV="${AOXC_Q_ENV:-testnet}"
AOXC_Q_PROFILE="${AOXC_Q_PROFILE:-testnet}"
AOXC_Q_NODE_COUNT="${AOXC_Q_NODE_COUNT:-7}"
AOXC_Q_ROUNDS="${AOXC_Q_ROUNDS:-200}"
AOXC_Q_START="${AOXC_Q_START:-1}"
AOXC_Q_SLEEP_SECS="${AOXC_Q_SLEEP_SECS:-1}"
AOXC_Q_FORCE="${AOXC_Q_FORCE:-0}"
AOXC_Q_ACTION="${AOXC_Q_ACTION:-up}"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

AOXC rolling devnet persistent local supervisor.

Actions:
  up (default)     provision and start persistent node loops
  provision        provision only; do not start loops
  start            start loops for an existing provisioned testnet root
  stop             stop loops for an existing provisioned testnet root
  restart          stop then start
  status           show per-node process and height status

Options:
  --action <name>      one of: up|provision|start|stop|restart|status
  --home <path>        base path for generated testnet root (default: ${AOXC_Q_HOME})
  --env <name>         configs/environments/<name> source (default: ${AOXC_Q_ENV})
  --profile <name>     AOXC profile for bootstrap (default: ${AOXC_Q_PROFILE})
  --nodes <n>          node count (default: ${AOXC_Q_NODE_COUNT}; minimum: 7)
  --rounds <n>         rounds per node-run cycle (default: ${AOXC_Q_ROUNDS})
  --sleep-secs <n>     sleep between cycles in daemon loop (default: ${AOXC_Q_SLEEP_SECS})
  --no-start           alias for --action provision
  --force              recreate existing testnet root during provision
  -h, --help           show this help

Environment overrides:
  AOXC_Q_HOME, AOXC_Q_ENV, AOXC_Q_PROFILE, AOXC_Q_NODE_COUNT,
  AOXC_Q_ROUNDS, AOXC_Q_START, AOXC_Q_SLEEP_SECS, AOXC_Q_FORCE, AOXC_Q_ACTION
USAGE
}

log_info() { printf '[aoxc-q][info] %s\n' "$*"; }
log_warn() { printf '[aoxc-q][warn] %s\n' "$*" >&2; }
log_error() { printf '[aoxc-q][error] %s\n' "$*" >&2; }

die() {
  local message="$1"
  local code="${2:-1}"
  log_error "${message}"
  exit "${code}"
}

require_uint() {
  local value="$1"
  local name="$2"
  [[ "${value}" =~ ^[0-9]+$ ]] || die "${name} must be an unsigned integer (got: ${value})" 2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --action) AOXC_Q_ACTION="$2"; shift 2 ;;
    --home) AOXC_Q_HOME="$2"; shift 2 ;;
    --home=*) AOXC_Q_HOME="${1#*=}"; shift ;;
    --env) AOXC_Q_ENV="$2"; shift 2 ;;
    --env=*) AOXC_Q_ENV="${1#*=}"; shift ;;
    --profile) AOXC_Q_PROFILE="$2"; shift 2 ;;
    --nodes) AOXC_Q_NODE_COUNT="$2"; shift 2 ;;
    --rounds) AOXC_Q_ROUNDS="$2"; shift 2 ;;
    --sleep-secs) AOXC_Q_SLEEP_SECS="$2"; shift 2 ;;
    --no-start) AOXC_Q_ACTION="provision"; AOXC_Q_START=0; shift ;;
    --force) AOXC_Q_FORCE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) die "Unknown argument: $1" 2 ;;
  esac
done

require_uint "${AOXC_Q_NODE_COUNT}" "AOXC_Q_NODE_COUNT"
require_uint "${AOXC_Q_ROUNDS}" "AOXC_Q_ROUNDS"
require_uint "${AOXC_Q_SLEEP_SECS}" "AOXC_Q_SLEEP_SECS"

if (( AOXC_Q_NODE_COUNT < 7 )); then
  die "AOXC_Q_NODE_COUNT must be >= 7 for production-like persistent topology." 2
fi
if (( AOXC_Q_SLEEP_MIN_SECS < 1 )); then
  die "AOXC_Q_SLEEP_MIN_SECS must be >= 1." 2
fi
if (( AOXC_Q_SLEEP_MAX_SECS < AOXC_Q_SLEEP_MIN_SECS )); then
  die "AOXC_Q_SLEEP_MAX_SECS must be >= AOXC_Q_SLEEP_MIN_SECS." 2
fi

case "${AOXC_Q_ACTION}" in
  up|provision|start|stop|restart|status) ;;
  *) die "Invalid --action value: ${AOXC_Q_ACTION}" 2 ;;
esac

case "${AOXC_Q_ACTION}" in
  up|provision|start|stop|restart|status) ;;
  *) die "Invalid --action value: ${AOXC_Q_ACTION}" 2 ;;
esac

TARGET_ROOT="${AOXC_Q_HOME%/}/aoxc-rolling-${AOXC_Q_ENV}-${AOXC_Q_NODE_COUNT}n"
SOURCE_ROOT="${REPO_ROOT}/configs/environments/${AOXC_Q_ENV}"

resolve_aoxc_command() {
  if [[ -x "${REPO_ROOT}/target/release/aoxc" ]]; then
    AOXC_CMD=("${REPO_ROOT}/target/release/aoxc")
  else
    AOXC_CMD=(cargo run -q -p aoxcmd --)
  fi
}

run_aoxc() {
  local home="$1"
  shift
  if [[ -n "${home}" ]]; then
    AOXC_HOME="${home}" "${AOXC_CMD[@]}" "$@"
  else
    "${AOXC_CMD[@]}" "$@"
  fi
}

write_wrapper_script() {
  local wrapper_path="$1"
  if [[ -x "${REPO_ROOT}/target/release/aoxc" ]]; then
    cat > "${wrapper_path}" <<WRAPPER
#!/usr/bin/env bash
set -Eeuo pipefail
exec "${REPO_ROOT}/target/release/aoxc" "\$@"
WRAPPER
  else
    cat > "${wrapper_path}" <<WRAPPER
#!/usr/bin/env bash
set -Eeuo pipefail
cd "${REPO_ROOT}"
exec cargo run -q -p aoxcmd -- "\$@"
WRAPPER
  fi
  chmod +x "${wrapper_path}"
}

required_source_files=(
  manifest.v1.json
  genesis.v1.json
  genesis.v1.sha256
  validators.json
  bootnodes.json
  certificate.json
  profile.toml
  release-policy.toml
)

prepare_directories() {
  mkdir -p \
    "${TARGET_ROOT}/system/genesis" \
    "${TARGET_ROOT}/system/config" \
    "${TARGET_ROOT}/system/audit" \
    "${TARGET_ROOT}/system/logs" \
    "${TARGET_ROOT}/system/scripts" \
    "${TARGET_ROOT}/nodes"
}

copy_environment_files() {
  cp "${SOURCE_ROOT}/manifest.v1.json" "${TARGET_ROOT}/system/genesis/manifest.json"
  cp "${SOURCE_ROOT}/genesis.v1.json" "${TARGET_ROOT}/system/genesis/genesis.json"
  cp "${SOURCE_ROOT}/genesis.v1.sha256" "${TARGET_ROOT}/system/genesis/genesis.sha256"
  cp "${SOURCE_ROOT}/validators.json" "${TARGET_ROOT}/system/config/validators.json"
  cp "${SOURCE_ROOT}/bootnodes.json" "${TARGET_ROOT}/system/config/bootnodes.json"
  cp "${SOURCE_ROOT}/certificate.json" "${TARGET_ROOT}/system/config/certificate.json"
  cp "${SOURCE_ROOT}/profile.toml" "${TARGET_ROOT}/system/config/profile.toml"
  cp "${SOURCE_ROOT}/release-policy.toml" "${TARGET_ROOT}/system/config/release-policy.toml"
}

render_node_runner() {
  local node_root="$1"
  local node_name="$2"
  local node_home="$3"
  local node_log="$4"
  local node_state_file="$5"

  cat > "${node_root}/run-node.sh" <<RUNNER
#!/usr/bin/env bash
set -Eeuo pipefail
NODE_NAME="${node_name}"
NODE_HOME="${node_home}"
ROUNDS="${AOXC_Q_ROUNDS}"
SLEEP_SECS="${AOXC_Q_SLEEP_SECS}"
WRAPPER="${TARGET_ROOT}/system/scripts/aoxc-wrapper.sh"
LOG_FILE="${node_log}"
STATE_FILE="${node_state_file}"

mkdir -p "\$(dirname "\${LOG_FILE}")"

while true; do
  ts_start="\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
  if AOXC_HOME="\${NODE_HOME}" "\${WRAPPER}" node-run --home "\${NODE_HOME}" --rounds "\${ROUNDS}" --tx-prefix "\${NODE_NAME^^}-TX" --format json --no-live-log >>"\${LOG_FILE}" 2>&1; then
    ts_end="\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
    printf '%s\tstatus=ok\tnode=%s\n' "\${ts_end}" "\${NODE_NAME}" > "\${STATE_FILE}"
  else
    ts_end="\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
    printf '%s\tstatus=error\tnode=%s\n' "\${ts_end}" "\${NODE_NAME}" > "\${STATE_FILE}"
  fi
  sleep "\${SLEEP_SECS}"
done
RUNNER
  chmod +x "${node_root}/run-node.sh"
}

provision_testnet() {
  [[ -d "${SOURCE_ROOT}" ]] || die "Missing source environment: ${SOURCE_ROOT}" 3
  for required in "${required_source_files[@]}"; do
    [[ -f "${SOURCE_ROOT}/${required}" ]] || die "Missing required source file: ${SOURCE_ROOT}/${required}" 3
  done

  if [[ "${AOXC_Q_FORCE}" == "1" && -e "${TARGET_ROOT}" ]]; then
    chmod -R u+w "${TARGET_ROOT}" 2>/dev/null || true
    chattr -R -i "${TARGET_ROOT}" 2>/dev/null || true
    rm -rf "${TARGET_ROOT}"
  fi

  if [[ -e "${TARGET_ROOT}" ]]; then
    die "Target already exists: ${TARGET_ROOT} (use --force to recreate)" 4
  fi

  prepare_directories
  copy_environment_files
  write_wrapper_script "${TARGET_ROOT}/system/scripts/aoxc-wrapper.sh"

  local accounts_file="${TARGET_ROOT}/system/audit/prepared-accounts.tsv"
  printf 'node\tvalidator_name\toperator_name\n' > "${accounts_file}"

  for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
    local node_name
    local validator_name
    local operator_name
    local password
    local node_root
    local node_home
    local run_dir
    local log_dir
    local state_file

    node_name="node$(printf '%02d' "${i}")"
    validator_name="aoxcdev-val-$(printf '%02d' "${i}")"
    operator_name="aoxcdev-op-$(printf '%02d' "${i}")"
    password="AOXC-ROLLING-${AOXC_Q_ENV^^}-${node_name^^}-CHANGE-ME"

    node_root="${TARGET_ROOT}/nodes/${node_name}"
    node_home="${node_root}/home"
    run_dir="${node_root}/run"
    log_dir="${node_root}/logs"
    state_file="${run_dir}/loop.state"

    mkdir -p "${node_home}/identity" "${node_home}/config" "${node_home}/runtime" "${node_home}/audit" "${run_dir}" "${log_dir}"

    cp "${TARGET_ROOT}/system/genesis/genesis.json" "${node_home}/identity/genesis.json"
    cp "${TARGET_ROOT}/system/config/profile.toml" "${node_home}/config/profile.toml"
    cp "${TARGET_ROOT}/system/config/validators.json" "${node_home}/config/validators.json"
    cp "${TARGET_ROOT}/system/config/bootnodes.json" "${node_home}/config/bootnodes.json"

    printf '%s\n' "${password}" > "${node_root}/operator.password"
    chmod 600 "${node_root}/operator.password"

    run_aoxc "${node_home}" config-init --profile "${AOXC_Q_PROFILE}" --json-logs > "${run_dir}/config-init.json"
    run_aoxc "${node_home}" address-create --name "${operator_name}" --profile "${AOXC_Q_PROFILE}" --password "${password}" > "${run_dir}/address-create-operator.json"
    run_aoxc "${node_home}" address-create --name "${validator_name}" --profile "${AOXC_Q_PROFILE}" --password "${password}" > "${run_dir}/address-create-validator.json"
    run_aoxc "${node_home}" key-bootstrap --profile "${AOXC_Q_PROFILE}" --name "${validator_name}" --password "${password}" > "${run_dir}/key-bootstrap.json"
    run_aoxc "${node_home}" keys-verify --password "${password}" > "${run_dir}/keys-verify.json"
    run_aoxc "" node-bootstrap --home "${node_home}" > "${run_dir}/node-bootstrap.json"

    render_node_runner "${node_root}" "${node_name}" "${node_home}" "${log_dir}/node-run.log" "${state_file}"

    printf '%s\t%s\t%s\n' "${node_name}" "${validator_name}" "${operator_name}" >> "${accounts_file}"
  done

  chmod -R go-rwx "${TARGET_ROOT}" || true
  chmod -R u+rwX "${TARGET_ROOT}" || true

  cat > "${TARGET_ROOT}/system/audit/provision-report.txt" <<REPORT
AOXC rolling devnet persistent local bootstrap report
created_utc=$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)
repo_root=${REPO_ROOT}
environment=${AOXC_Q_ENV}
profile=${AOXC_Q_PROFILE}
mode=${AOXC_Q_MODE}
node_count=${AOXC_Q_NODE_COUNT}
rounds=${AOXC_Q_ROUNDS}
sleep_secs=${AOXC_Q_SLEEP_SECS}
root=${TARGET_ROOT}
REPORT

  log_info "Provisioning complete: ${TARGET_ROOT}"
}

start_testnet() {
  [[ -d "${TARGET_ROOT}/nodes" ]] || die "Target root is not provisioned: ${TARGET_ROOT}" 5

  for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
    local node_name
    local node_root
    local pid_file
    node_name="node$(printf '%02d' "${i}")"
    node_root="${TARGET_ROOT}/nodes/${node_name}"
    pid_file="${node_root}/node.pid"

    if [[ -f "${pid_file}" ]]; then
      local pid
      pid="$(cat "${pid_file}")"
      if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
        log_info "${node_name} already running pid=${pid}"
        continue
      fi
      rm -f "${pid_file}"
    fi

    nohup "${node_root}/run-node.sh" >"${node_root}/logs/supervisor.log" 2>&1 &
    echo "$!" > "${pid_file}"
    log_info "started ${node_name} pid=$(cat "${pid_file}")"
  done
}

stop_testnet() {
  [[ -d "${TARGET_ROOT}/nodes" ]] || die "Target root is not provisioned: ${TARGET_ROOT}" 5

  for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
    local node_name
    local node_root
    local pid_file
    node_name="node$(printf '%02d' "${i}")"
    node_root="${TARGET_ROOT}/nodes/${node_name}"
    pid_file="${node_root}/node.pid"

    [[ -f "${pid_file}" ]] || { log_info "${node_name} not running"; continue; }

    local pid
    pid="$(cat "${pid_file}")"
    if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" || true
      sleep 1
      if kill -0 "${pid}" 2>/dev/null; then
        kill -9 "${pid}" || true
      fi
      log_info "stopped ${node_name} pid=${pid}"
    else
      log_warn "${node_name} pid file stale (${pid})"
    fi
    rm -f "${pid_file}"
  done
}

status_testnet() {
  [[ -d "${TARGET_ROOT}/nodes" ]] || die "Target root is not provisioned: ${TARGET_ROOT}" 5
  printf 'node\tprocess\tpid\theight\tupdated_at\n'

  for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
    local node_name
    local node_root
    local pid_file
    local state_json
    local process_state
    local pid_text
    local height
    local updated_at

    node_name="node$(printf '%02d' "${i}")"
    node_root="${TARGET_ROOT}/nodes/${node_name}"
    pid_file="${node_root}/node.pid"
    process_state="stopped"
    pid_text="-"

    if [[ -f "${pid_file}" ]]; then
      pid_text="$(cat "${pid_file}")"
      if [[ -n "${pid_text}" ]] && kill -0 "${pid_text}" 2>/dev/null; then
        process_state="running"
      else
        process_state="stale-pid"
      fi
    fi

    height="-"
    updated_at="-"
    if state_json="$(run_aoxc "${node_root}/home" chain-status --format json 2>/dev/null)"; then
      height="$(printf '%s' "${state_json}" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("current_height","-"))' 2>/dev/null || echo '-')"
      updated_at="$(printf '%s' "${state_json}" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("updated_at","-"))' 2>/dev/null || echo '-')"
    fi

    printf '%s\t%s\t%s\t%s\t%s\n' "${node_name}" "${process_state}" "${pid_text}" "${height}" "${updated_at}"
  done
}

main() {
  resolve_aoxc_command

  case "${AOXC_Q_ACTION}" in
    up)
      provision_testnet
      start_testnet
      ;;
    provision)
      provision_testnet
      ;;
    start)
      start_testnet
      ;;
    stop)
      stop_testnet
      ;;
    restart)
      stop_testnet
      start_testnet
      ;;
    status)
      status_testnet
      ;;
  esac

  if [[ "${AOXC_Q_ACTION}" == "up" || "${AOXC_Q_ACTION}" == "provision" ]]; then
    log_info "accounts: ${TARGET_ROOT}/system/audit/prepared-accounts.tsv"
    log_info "provision report: ${TARGET_ROOT}/system/audit/provision-report.txt"
    log_info "control: $(basename "$0") --action start|stop|status --home ${AOXC_Q_HOME} --env ${AOXC_Q_ENV} --nodes ${AOXC_Q_NODE_COUNT}"
  fi
}

main "$@"
