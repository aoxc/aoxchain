#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SCRIPT_VERSION="2026.04.06-r2"

AOXC_Q_HOME="${AOXC_Q_HOME:-/mnt/xdbx/aoxc}"
AOXC_Q_ENV="${AOXC_Q_ENV:-testnet}"
AOXC_Q_PROFILE="${AOXC_Q_PROFILE:-testnet}"
AOXC_Q_NODE_COUNT="${AOXC_Q_NODE_COUNT:-7}"
AOXC_Q_ROUNDS="${AOXC_Q_ROUNDS:-200}"
AOXC_Q_SLEEP_MS="${AOXC_Q_SLEEP_MS:-250}"
AOXC_Q_FORCE="${AOXC_Q_FORCE:-0}"
AOXC_Q_PASSWORD="${AOXC_Q_PASSWORD:-}"
AOXC_Q_PASSWORD_FILE="${AOXC_Q_PASSWORD_FILE:-}"
AOXC_Q_PROMPT_PASSWORD="${AOXC_Q_PROMPT_PASSWORD:-1}"

AOXC_Q_COMMAND="up"

usage() {
  cat <<USAGE
Usage:
  $(basename "$0") <command> [options]

Commands:
  up         bootstrap + start all nodes (default command)
  bootstrap  only prepare and bootstrap node homes
  start      start all node runner loops
  stop       stop all running node loops
  status     show per-node pid/running status
  logs       tail a specific node log
  help       show this help

Common options:
  --home <path>         Root path (default: ${AOXC_Q_HOME})
  --env <name>          configs/environments/<name> source (default: ${AOXC_Q_ENV})
  --profile <name>      AOXC profile for config/key bootstrap (default: ${AOXC_Q_PROFILE})
  --nodes <n>           node count, minimum 7 (default: ${AOXC_Q_NODE_COUNT})
  --rounds <n>          rounds per node-run invocation (default: ${AOXC_Q_ROUNDS})
  --sleep-ms <n>        sleep between rounds in ms (default: ${AOXC_Q_SLEEP_MS})
  --no-start            alias for command=bootstrap (backward compatibility)
  --force               remove existing target root during bootstrap

Password options (bootstrap/up only):
  --password <value>    operator password for all nodes
  --password-file <p>   read password from first line of file
  --prompt-password     prompt for password when not provided
  --no-prompt-password  fail if password is not provided

Logs command options:
  --node <name>         node id (e.g. node01)
  --lines <n>           tail line count (default: 120)

Environment:
  AOXC_Q_HOME AOXC_Q_ENV AOXC_Q_PROFILE AOXC_Q_NODE_COUNT AOXC_Q_ROUNDS
  AOXC_Q_SLEEP_MS AOXC_Q_FORCE AOXC_Q_PASSWORD AOXC_Q_PASSWORD_FILE
  AOXC_Q_PROMPT_PASSWORD
USAGE
}

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 2
}

require_uint() {
  local value="$1"
  local name="$2"
  [[ "${value}" =~ ^[0-9]+$ ]] || die "${name} must be an unsigned integer (got: ${value})"
}

run_cmd_logged() {
  local log_file="$1"
  shift
  mkdir -p "$(dirname "${log_file}")"
  if ! "$@" >"${log_file}" 2>&1; then
    echo "Command failed. Log: ${log_file}" >&2
    tail -n 40 "${log_file}" >&2 || true
    exit 10
  fi
}

resolve_password() {
  if [[ -n "${AOXC_Q_PASSWORD}" ]]; then
    return 0
  fi

  if [[ -n "${AOXC_Q_PASSWORD_FILE}" ]]; then
    [[ -f "${AOXC_Q_PASSWORD_FILE}" ]] || die "Password file does not exist: ${AOXC_Q_PASSWORD_FILE}"
    AOXC_Q_PASSWORD="$(head -n 1 "${AOXC_Q_PASSWORD_FILE}")"
  fi

  if [[ -z "${AOXC_Q_PASSWORD}" && "${AOXC_Q_PROMPT_PASSWORD}" == "1" ]]; then
    [[ -t 0 ]] || die "Cannot prompt in non-interactive shell. Use --password or --password-file."
    read -r -s -p "AOXC operator password: " AOXC_Q_PASSWORD
    echo ""
  fi

  [[ -n "${AOXC_Q_PASSWORD}" ]] || die "Missing operator password. Use --password, --password-file, or AOXC_Q_PASSWORD."
}

source_root() {
  printf '%s\n' "${REPO_ROOT}/configs/environments/${AOXC_Q_ENV}"
}

target_root() {
  printf '%s\n' "${AOXC_Q_HOME%/}/aoxc-q-v0.2.0-${AOXC_Q_ENV}-${AOXC_Q_NODE_COUNT}n"
}

node_name() {
  local i="$1"
  printf 'node%02d\n' "${i}"
}

resolve_bin_cmd() {
  if [[ -x "${REPO_ROOT}/target/release/aoxc" ]]; then
    AOXC_BIN_CMD=("${REPO_ROOT}/target/release/aoxc")
  else
    AOXC_BIN_CMD=(cargo run -q -p aoxcmd --)
  fi
}

validate_common_inputs() {
  require_uint "${AOXC_Q_NODE_COUNT}" "AOXC_Q_NODE_COUNT"
  require_uint "${AOXC_Q_ROUNDS}" "AOXC_Q_ROUNDS"
  require_uint "${AOXC_Q_SLEEP_MS}" "AOXC_Q_SLEEP_MS"
  (( AOXC_Q_NODE_COUNT >= 7 )) || die "AOXC_Q_NODE_COUNT must be >= 7"
}

validate_source_bundle() {
  local src
  src="$(source_root)"
  [[ -d "${src}" ]] || die "Missing source environment: ${src}"
  for required in manifest.v1.json genesis.v1.json genesis.v1.sha256 validators.json bootnodes.json certificate.json profile.toml release-policy.toml; do
    [[ -f "${src}/${required}" ]] || die "Missing required source file: ${src}/${required}"
  done
}

prepare_system_layout() {
  local src root
  src="$(source_root)"
  root="$(target_root)"

  if [[ "${AOXC_Q_FORCE}" == "1" && -e "${root}" ]]; then
    chmod -R u+w "${root}" 2>/dev/null || true
    chattr -R -i "${root}" 2>/dev/null || true
    rm -rf "${root}"
  fi

  [[ ! -e "${root}" ]] || die "Target already exists: ${root} (use --force)"

  mkdir -p \
    "${root}/system/genesis" \
    "${root}/system/config" \
    "${root}/system/audit" \
    "${root}/system/logs" \
    "${root}/system/scripts" \
    "${root}/nodes"

  cp "${src}/manifest.v1.json" "${root}/system/genesis/manifest.json"
  cp "${src}/genesis.v1.json" "${root}/system/genesis/genesis.json"
  cp "${src}/genesis.v1.sha256" "${root}/system/genesis/genesis.sha256"
  cp "${src}/validators.json" "${root}/system/config/validators.json"
  cp "${src}/bootnodes.json" "${root}/system/config/bootnodes.json"
  cp "${src}/certificate.json" "${root}/system/config/certificate.json"
  cp "${src}/profile.toml" "${root}/system/config/profile.toml"
  cp "${src}/release-policy.toml" "${root}/system/config/release-policy.toml"

  printf 'node\tvalidator_name\n' > "${root}/system/audit/prepared-accounts.tsv"
}

write_runner() {
  local node_root="$1"
  local node_home="$2"
  local node_log="$3"
  local node_tx_prefix="$4"

  cat > "${node_root}/run-node.sh" <<RUNNER
#!/usr/bin/env bash
set -Eeuo pipefail
AOXC_HOME="${node_home}"
AOXC_BIN="${AOXC_BIN_CMD[0]}"
while true; do
  "\${AOXC_BIN}" node-run --home "\${AOXC_HOME}" --rounds "${AOXC_Q_ROUNDS}" --sleep-ms "${AOXC_Q_SLEEP_MS}" --tx-prefix "${node_tx_prefix}" >> "${node_log}" 2>&1 || true
  sleep 1
done
RUNNER
  chmod +x "${node_root}/run-node.sh"
}

bootstrap_nodes() {
  local root
  root="$(target_root)"

  for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
    local name validator node_root node_home run_dir log_dir
    name="$(node_name "${i}")"
    validator="aoxcq-val-$(printf '%02d' "${i}")"
    node_root="${root}/nodes/${name}"
    node_home="${node_root}/home"
    run_dir="${node_root}/run"
    log_dir="${node_root}/logs"

    mkdir -p "${node_home}/identity" "${node_home}/config" "${node_home}/runtime" "${node_home}/audit" "${run_dir}" "${log_dir}"

    cp "${root}/system/genesis/genesis.json" "${node_home}/identity/genesis.json"
    cp "${root}/system/config/profile.toml" "${node_home}/config/profile.toml"
    cp "${root}/system/config/validators.json" "${node_home}/config/validators.json"
    cp "${root}/system/config/bootnodes.json" "${node_home}/config/bootnodes.json"

    printf '%s\n' "${AOXC_Q_PASSWORD}" > "${node_root}/operator.password"
    chmod 600 "${node_root}/operator.password" || true

    run_cmd_logged "${run_dir}/config-init.log" env AOXC_HOME="${node_home}" "${AOXC_BIN_CMD[@]}" config-init --profile "${AOXC_Q_PROFILE}" --json-logs
    run_cmd_logged "${run_dir}/key-bootstrap.log" env AOXC_HOME="${node_home}" "${AOXC_BIN_CMD[@]}" key-bootstrap --profile "${AOXC_Q_PROFILE}" --name "${validator}" --password "${AOXC_Q_PASSWORD}" --force
    run_cmd_logged "${run_dir}/keys-verify.log" env AOXC_HOME="${node_home}" "${AOXC_BIN_CMD[@]}" keys-verify --password "${AOXC_Q_PASSWORD}"
    run_cmd_logged "${run_dir}/node-bootstrap.log" "${AOXC_BIN_CMD[@]}" node-bootstrap --home "${node_home}"

    printf '%s\t%s\n' "${name}" "${validator}" >> "${root}/system/audit/prepared-accounts.tsv"
    write_runner "${node_root}" "${node_home}" "${log_dir}/node-run.log" "${name^^}-TX"
  done
}

write_control_scripts() {
  local root
  root="$(target_root)"

  # replace placeholder with stable shell content without nested interpolation issues
  cat > "${root}/system/scripts/start-all.sh" <<'STARTALL'
#!/usr/bin/env bash
set -Eeuo pipefail
ROOT_PLACEHOLDER="__ROOT__"
ROOT="${ROOT_PLACEHOLDER}"
for node_root in "${ROOT}"/nodes/node*; do
  [[ -d "${node_root}" ]] || continue
  node_name="$(basename "${node_root}")"
  if [[ -f "${node_root}/node.pid" ]] && kill -0 "$(cat "${node_root}/node.pid")" 2>/dev/null; then
    echo "[skip] ${node_name} already running"
    continue
  fi
  nohup "${node_root}/run-node.sh" >/dev/null 2>&1 &
  echo $! > "${node_root}/node.pid"
  echo "[ok] started ${node_name} pid=$(cat "${node_root}/node.pid")"
done
STARTALL
  sed -i "s|__ROOT__|${root}|g" "${root}/system/scripts/start-all.sh"

  cat > "${root}/system/scripts/stop-all.sh" <<'STOPALL'
#!/usr/bin/env bash
set -Eeuo pipefail
ROOT_PLACEHOLDER="__ROOT__"
ROOT="${ROOT_PLACEHOLDER}"
for node_root in "${ROOT}"/nodes/node*; do
  [[ -d "${node_root}" ]] || continue
  node_name="$(basename "${node_root}")"
  if [[ -f "${node_root}/node.pid" ]]; then
    pid="$(cat "${node_root}/node.pid")"
    if kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" || true
      echo "[ok] stopped ${node_name} pid=${pid}"
    fi
    rm -f "${node_root}/node.pid"
  fi
done
STOPALL
  sed -i "s|__ROOT__|${root}|g" "${root}/system/scripts/stop-all.sh"

  cat > "${root}/system/scripts/status-all.sh" <<'STATUSALL'
#!/usr/bin/env bash
set -Eeuo pipefail
ROOT_PLACEHOLDER="__ROOT__"
ROOT="${ROOT_PLACEHOLDER}"
for node_root in "${ROOT}"/nodes/node*; do
  [[ -d "${node_root}" ]] || continue
  node_name="$(basename "${node_root}")"
  if [[ -f "${node_root}/node.pid" ]] && kill -0 "$(cat "${node_root}/node.pid")" 2>/dev/null; then
    echo "${node_name}: running pid=$(cat "${node_root}/node.pid")"
  else
    echo "${node_name}: stopped"
  fi
done
STATUSALL
  sed -i "s|__ROOT__|${root}|g" "${root}/system/scripts/status-all.sh"

  chmod +x "${root}/system/scripts/start-all.sh" "${root}/system/scripts/stop-all.sh" "${root}/system/scripts/status-all.sh"
}

write_report() {
  local root
  root="$(target_root)"

  cat > "${root}/system/audit/provision-report.txt" <<REPORT
AOXC-Q v0.2.0 local testnet bootstrap report
script_version=${SCRIPT_VERSION}
created_utc=$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)
repo_root=${REPO_ROOT}
environment=${AOXC_Q_ENV}
profile=${AOXC_Q_PROFILE}
node_count=${AOXC_Q_NODE_COUNT}
rounds=${AOXC_Q_ROUNDS}
sleep_ms=${AOXC_Q_SLEEP_MS}
root=${root}
REPORT

  chmod -R go-rwx "${root}" || true
  chmod -R u+rwX "${root}" || true
}

start_nodes() {
  local root
  root="$(target_root)"
  [[ -x "${root}/system/scripts/start-all.sh" ]] || die "Missing start script. Run bootstrap first."
  "${root}/system/scripts/start-all.sh"
}

stop_nodes() {
  local root
  root="$(target_root)"
  [[ -x "${root}/system/scripts/stop-all.sh" ]] || die "Missing stop script. Run bootstrap first."
  "${root}/system/scripts/stop-all.sh"
}

status_nodes() {
  local root
  root="$(target_root)"
  [[ -x "${root}/system/scripts/status-all.sh" ]] || die "Missing status script. Run bootstrap first."
  "${root}/system/scripts/status-all.sh"
}

logs_node=""
logs_lines="120"

tail_logs() {
  local root node file
  root="$(target_root)"
  [[ -n "${logs_node}" ]] || die "logs requires --node <name>"
  require_uint "${logs_lines}" "--lines"
  file="${root}/nodes/${logs_node}/logs/node-run.log"
  [[ -f "${file}" ]] || die "Log file not found: ${file}"
  tail -n "${logs_lines}" "${file}"
}

parse_args() {
  if [[ $# -gt 0 ]]; then
    case "$1" in
      up|bootstrap|start|stop|status|logs|help)
        AOXC_Q_COMMAND="$1"
        shift
        ;;
    esac
  fi

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --home) AOXC_Q_HOME="$2"; shift 2 ;;
      --env) AOXC_Q_ENV="$2"; shift 2 ;;
      --profile) AOXC_Q_PROFILE="$2"; shift 2 ;;
      --nodes) AOXC_Q_NODE_COUNT="$2"; shift 2 ;;
      --rounds) AOXC_Q_ROUNDS="$2"; shift 2 ;;
      --sleep-ms) AOXC_Q_SLEEP_MS="$2"; shift 2 ;;
      --no-start) AOXC_Q_COMMAND="bootstrap"; shift ;;
      --force) AOXC_Q_FORCE=1; shift ;;
      --password) AOXC_Q_PASSWORD="$2"; shift 2 ;;
      --password-file) AOXC_Q_PASSWORD_FILE="$2"; shift 2 ;;
      --prompt-password) AOXC_Q_PROMPT_PASSWORD=1; shift ;;
      --no-prompt-password) AOXC_Q_PROMPT_PASSWORD=0; shift ;;
      --node) logs_node="$2"; shift 2 ;;
      --lines) logs_lines="$2"; shift 2 ;;
      -h|--help) AOXC_Q_COMMAND="help"; shift ;;
      *) die "Unknown argument: $1" ;;
    esac
  done
}

main() {
  parse_args "$@"

  if [[ "${AOXC_Q_COMMAND}" == "help" ]]; then
    usage
    exit 0
  fi

  validate_common_inputs
  resolve_bin_cmd

  case "${AOXC_Q_COMMAND}" in
    up)
      validate_source_bundle
      resolve_password
      prepare_system_layout
      bootstrap_nodes
      write_control_scripts
      write_report
      start_nodes
      ;;
    bootstrap)
      validate_source_bundle
      resolve_password
      prepare_system_layout
      bootstrap_nodes
      write_control_scripts
      write_report
      ;;
    start)
      start_nodes
      ;;
    stop)
      stop_nodes
      ;;
    status)
      status_nodes
      ;;
    logs)
      tail_logs
      ;;
    *)
      die "Unknown command: ${AOXC_Q_COMMAND}"
      ;;
  esac

  if [[ "${AOXC_Q_COMMAND}" == "up" || "${AOXC_Q_COMMAND}" == "bootstrap" ]]; then
    local root
    root="$(target_root)"
    echo "AOXC-Q ${AOXC_Q_COMMAND} completed."
    echo "root: ${root}"
    echo "accounts: ${root}/system/audit/prepared-accounts.tsv"
    echo "bootstrap logs: ${root}/nodes/nodeXX/run/*.log"
    echo "start: ${root}/system/scripts/start-all.sh"
    echo "stop:  ${root}/system/scripts/stop-all.sh"
    echo "status:${root}/system/scripts/status-all.sh"
  fi
}

main "$@"
