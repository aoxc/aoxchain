#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Execute a controlled distributed validation workflow across 3 to 5 remote
#   hosts using a deterministic fixture set.
#
# Operational Intent:
#   - Synchronize a canonical fixture set to remote operators
#   - Bootstrap deterministic node homes on each target host
#   - Execute bounded node runs with deterministic transaction prefixes
#   - Preserve per-node execution evidence under a local artifact directory
#   - Avoid assumptions regarding root access, traffic shaping, or fault
#     injection primitives such as netem or iptables
#
# Security and Reliability Posture:
#   - Fail closed on missing prerequisites and malformed host inventory
#   - Avoid unsafe shell evaluation patterns
#   - Keep remote command construction explicit and reviewable
#   - Emit machine-reviewable logs and a human-oriented summary scaffold
#
# Notes:
#   - The host inventory must contain one reachable host per line
#   - Comment lines beginning with '#' and blank lines are ignored
#   - The first five usable hosts are mapped deterministically to:
#       atlas, boreal, cypher, delta, ember
#
# Exit Codes:
#   0  Successful completion
#   1  General validation failure
#   2  Invalid configuration or invocation
#   3  Missing required dependency or runtime binary
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

readonly DEFAULT_HOSTS_FILE="${ROOT_DIR}/configs/deterministic-testnet/hosts.txt"
readonly DEFAULT_HOSTS_TEMPLATE="${ROOT_DIR}/configs/deterministic-testnet/hosts.txt.example"
readonly DEFAULT_REMOTE_BASE="aoxc-distributed"
readonly DEFAULT_FIXTURE_DIR="${ROOT_DIR}/configs/deterministic-testnet"
readonly DEFAULT_ARTIFACT_DIR="${ROOT_DIR}/artifacts/distributed-validation"
readonly DEFAULT_ROUNDS=5
readonly DEFAULT_SLEEP_MS=250

readonly NODE_NAMES=(
  "atlas"
  "boreal"
  "cypher"
  "delta"
  "ember"
)

HOSTS_FILE="${HOSTS_FILE:-$DEFAULT_HOSTS_FILE}"
HOSTS_TEMPLATE="${HOSTS_TEMPLATE:-$DEFAULT_HOSTS_TEMPLATE}"
REMOTE_BASE="${REMOTE_BASE:-$DEFAULT_REMOTE_BASE}"
FIXTURE_DIR="${FIXTURE_DIR:-$DEFAULT_FIXTURE_DIR}"
ARTIFACT_DIR="${ARTIFACT_DIR:-$DEFAULT_ARTIFACT_DIR}"
ROUNDS="${ROUNDS:-$DEFAULT_ROUNDS}"
SLEEP_MS="${SLEEP_MS:-$DEFAULT_SLEEP_MS}"

SSH_OPTIONS=(
  -o BatchMode=yes
  -o ConnectTimeout=10
)

RSYNC_OPTIONS=(
  -a
  -z
)

TMP_DIR=''
SANITIZED_HOSTS_FILE=''
SUMMARY_FILE=''
REPORT_FILE=''
INDEX_FILE=''
FAILURE_FILE=''
RUN_STARTED_AT_UTC=''

SUCCESS_COUNT=0
FAILURE_COUNT=0

log_info() {
  printf '[info] %s\n' "$*"
}

log_stage() {
  printf '[stage] %s\n' "$*"
}

log_warn() {
  printf '[warn] %s\n' "$*" >&2
}

log_error() {
  printf '[error] %s\n' "$*" >&2
}

fail() {
  local message="$1"
  log_error "${message}"
  exit 1
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || fail "Missing required command: ${command_name}"
}

require_file() {
  local file_path="$1"
  [[ -f "${file_path}" ]] || fail "Missing required file: ${file_path}"
}

require_directory() {
  local dir_path="$1"
  [[ -d "${dir_path}" ]] || fail "Missing required directory: ${dir_path}"
}

ensure_directory() {
  local dir_path="$1"

  if [[ -e "${dir_path}" && ! -d "${dir_path}" ]]; then
    fail "Path exists but is not a directory: ${dir_path}"
  fi

  mkdir -p "${dir_path}"
}

cleanup() {
  if [[ -n "${TMP_DIR}" && -d "${TMP_DIR}" ]]; then
    rm -rf "${TMP_DIR}"
  fi
}

on_error() {
  local exit_code="$1"
  local failed_line="$2"

  ensure_directory "${ARTIFACT_DIR}"

  cat > "${FAILURE_FILE}" <<EOF
timestamp_utc=${RUN_STARTED_AT_UTC}
status=failed
exit_code=${exit_code}
failed_line=${failed_line}
hosts_file=${HOSTS_FILE}
fixture_dir=${FIXTURE_DIR}
artifact_dir=${ARTIFACT_DIR}
rounds=${ROUNDS}
sleep_ms=${SLEEP_MS}
success_count=${SUCCESS_COUNT}
failure_count=${FAILURE_COUNT}
EOF

  log_error "Distributed validation failed at line ${failed_line} with exit code ${exit_code}"
}

trap 'on_error "$?" "$LINENO"' ERR
trap cleanup EXIT

validate_non_negative_integer() {
  local value="$1"
  local name="$2"

  [[ "${value}" =~ ^[0-9]+$ ]] || fail "${name} must be a non-negative integer"
}

resolve_bin_path() {
  # The binary resolution order is intentionally explicit to preserve
  # predictable operator behavior across packaged and repository-local
  # runtime layouts.
  if [[ -n "${BIN_PATH:-}" ]]; then
    [[ -x "${BIN_PATH}" ]] || fail "BIN_PATH is set but not executable: ${BIN_PATH}"
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

  if command -v aoxc >/dev/null 2>&1; then
    command -v aoxc
    return 0
  fi

  return 1
}

sanitize_hosts_file() {
  awk '
    /^[[:space:]]*#/ { next }
    /^[[:space:]]*$/ { next }
    { print }
  ' "${HOSTS_FILE}" > "${SANITIZED_HOSTS_FILE}"
}

validate_host_count() {
  local count="$1"

  if (( count < 3 )); then
    fail "At least 3 hosts must be provided in ${HOSTS_FILE}"
  fi

  if (( count > 5 )); then
    fail "At most 5 hosts are supported by this deterministic harness"
  fi
}

validate_host_line() {
  local host="$1"

  # The harness accepts conventional SSH targets such as:
  # - hostname
  # - hostname.domain
  # - user@hostname
  # - user@hostname.domain
  # - IPv4 literals
  # - user@IPv4 literals
  #
  # Whitespace and shell metacharacters are rejected explicitly to keep the
  # remote execution surface predictable and auditable.
  [[ "${host}" =~ ^[A-Za-z0-9._@:-]+$ ]] || fail "Invalid host entry detected: ${host}"
}

escape_json_string() {
  local raw="${1:-}"
  raw="${raw//\\/\\\\}"
  raw="${raw//\"/\\\"}"
  raw="${raw//$'\n'/\\n}"
  raw="${raw//$'\r'/\\r}"
  raw="${raw//$'\t'/\\t}"
  printf '%s' "${raw}"
}

escape_shell_single_quotes() {
  local raw="${1:-}"
  raw="${raw//\'/\'\\\'\'}"
  printf '%s' "${raw}"
}

remote_base_expanded() {
  if [[ "${REMOTE_BASE}" == "~/"* ]]; then
    printf '%s/%s\n' '$HOME' "${REMOTE_BASE#~/}"
    return 0
  fi

  if [[ "${REMOTE_BASE}" == "~" ]]; then
    printf '%s\n' '$HOME'
    return 0
  fi

  printf '%s\n' "${REMOTE_BASE}"
}

write_report_scaffold() {
  cat > "${REPORT_FILE}" <<EOF
# AOXC Distributed Validation Report

- generated_at_utc: ${RUN_STARTED_AT_UTC}
- hosts_file: ${HOSTS_FILE}
- fixture_dir: ${FIXTURE_DIR}
- artifact_dir: ${ARTIFACT_DIR}
- rounds: ${ROUNDS}
- sleep_ms: ${SLEEP_MS}

## Objective
Validate deterministic multi-host AOXC node bootstrap and bounded execution across a controlled distributed fixture.

## Environment
Document the exact remote hosts, operating system versions, available tooling, and any operator-specific SSH constraints.

## Procedure
1. Validate host inventory and local prerequisites.
2. Synchronize deterministic fixture content to each remote host.
3. Bootstrap one deterministic node home per host.
4. Execute a bounded run on each node using deterministic transaction prefixes.
5. Review bootstrap, run, and summary artifacts.

## Observations
Record successful node starts, bounded execution behavior, timing observations, and any divergence between hosts.

## Issues
Record bootstrap failures, remote path issues, binary availability problems, connectivity issues, or inconsistent output.

## Conclusion
State whether the distributed validation objective passed, failed, or requires rerun.

## Next Action
Describe the next engineering or operational action required for closure.
EOF
}

write_summary_file() {
  cat > "${SUMMARY_FILE}" <<EOF
run_started_at_utc=${RUN_STARTED_AT_UTC}
hosts_file=${HOSTS_FILE}
fixture_dir=${FIXTURE_DIR}
artifact_dir=${ARTIFACT_DIR}
rounds=${ROUNDS}
sleep_ms=${SLEEP_MS}
success_count=${SUCCESS_COUNT}
failure_count=${FAILURE_COUNT}
EOF
}

write_artifact_index() {
  local artifact_file=''
  local artifact_name=''
  local artifact_size=''
  local first=1

  {
    printf '{\n'
    printf '  "generated_at_utc": "%s",\n' "$(escape_json_string "${RUN_STARTED_AT_UTC}")"
    printf '  "artifact_dir": "%s",\n' "$(escape_json_string "${ARTIFACT_DIR}")"
    printf '  "success_count": %d,\n' "${SUCCESS_COUNT}"
    printf '  "failure_count": %d,\n' "${FAILURE_COUNT}"
    printf '  "artifacts": [\n'

    while IFS= read -r -d '' artifact_file; do
      artifact_name="$(basename "${artifact_file}")"
      artifact_size="$(stat -c '%s' "${artifact_file}")"

      if (( first == 0 )); then
        printf ',\n'
      fi
      first=0

      printf '    {\n'
      printf '      "name": "%s",\n' "$(escape_json_string "${artifact_name}")"
      printf '      "path": "%s",\n' "$(escape_json_string "${artifact_file}")"
      printf '      "size_bytes": %s\n' "${artifact_size}"
      printf '    }'
    done < <(find "${ARTIFACT_DIR}" -maxdepth 1 -type f -print0 | sort -z)

    if (( first == 0 )); then
      printf '\n'
    fi

    printf '  ]\n'
    printf '}\n'
  } > "${INDEX_FILE}"
}

ssh_run() {
  local host="$1"
  local command="$2"

  ssh "${SSH_OPTIONS[@]}" "${host}" "${command}"
}

rsync_push_fixture() {
  local host="$1"
  local remote_target="$2"

  rsync "${RSYNC_OPTIONS[@]}" "${FIXTURE_DIR}/" "${host}:${remote_target}/fixture/"
}

prepare_remote_host() {
  local host="$1"
  local remote_root="$2"

  ssh_run "${host}" "mkdir -p '${remote_root}/fixture' '${remote_root}/logs' '${remote_root}/home'"
}

run_remote_bootstrap() {
  local host="$1"
  local remote_root="$2"
  local node_name="$3"
  local rounds="$4"
  local sleep_ms="$5"
  local bin_path="$6"

  local remote_home="${remote_root}/home/${node_name}"
  local remote_log="${remote_root}/logs/${node_name}.log"
  local remote_tx_prefix="DIST_${node_name^^}"

  ssh_run "${host}" "\
set -Eeuo pipefail; \
export AOXC_HOME='${remote_home}'; \
mkdir -p '${remote_home}'; \
'$(escape_shell_single_quotes "${bin_path}")' node-bootstrap --home '${remote_home}' >> '${remote_log}' 2>&1; \
'$(escape_shell_single_quotes "${bin_path}")' node-run --home '${remote_home}' --rounds '${rounds}' --sleep-ms '${sleep_ms}' --tx-prefix '${remote_tx_prefix}' >> '${remote_log}' 2>&1"
}

collect_remote_artifacts() {
  local host="$1"
  local remote_root="$2"
  local node_name="$3"

  rsync "${RSYNC_OPTIONS[@]}" \
    "${host}:${remote_root}/logs/${node_name}.log" \
    "${ARTIFACT_DIR}/${node_name}.log"
}

main() {
  local host_count=0
  local host=''
  local node_name=''
  local node_index=0
  local bin_path=''
  local remote_root=''

  require_command awk
  require_command ssh
  require_command rsync
  require_command mktemp
  require_command stat
  require_command find
  require_command sort
  require_command basename
  require_command date

  validate_non_negative_integer "${ROUNDS}" "ROUNDS"
  validate_non_negative_integer "${SLEEP_MS}" "SLEEP_MS"

  require_file "${HOSTS_FILE}"
  require_file "${HOSTS_TEMPLATE}"
  require_directory "${FIXTURE_DIR}"

  if ! bin_path="$(resolve_bin_path)"; then
    fail "Unable to locate an executable AOXC binary. Build or install it with: make package-bin"
  fi

  ensure_directory "${ARTIFACT_DIR}"

  TMP_DIR="$(mktemp -d)"
  SANITIZED_HOSTS_FILE="${TMP_DIR}/hosts.cleaned"
  RUN_STARTED_AT_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  SUMMARY_FILE="${ARTIFACT_DIR}/summary.txt"
  REPORT_FILE="${ARTIFACT_DIR}/distributed-validation-report.md"
  INDEX_FILE="${ARTIFACT_DIR}/artifact-index.json"
  FAILURE_FILE="${ARTIFACT_DIR}/failure.log"

  remote_root="$(remote_base_expanded)"

  log_stage "Validating host inventory"
  sanitize_hosts_file
  host_count="$(wc -l < "${SANITIZED_HOSTS_FILE}")"
  validate_host_count "${host_count}"

  while IFS= read -r host; do
    validate_host_line "${host}"
  done < "${SANITIZED_HOSTS_FILE}"

  log_stage "Writing local report scaffold"
  write_report_scaffold

  log_stage "Executing distributed validation"
  while IFS= read -r host; do
    node_name="${NODE_NAMES[${node_index}]}"
    node_index=$((node_index + 1))

    log_info "Preparing remote host '${host}' for node '${node_name}'"
    prepare_remote_host "${host}" "${remote_root}"

    log_info "Synchronizing fixture set to '${host}'"
    rsync_push_fixture "${host}" "${remote_root}"

    log_info "Running bootstrap and bounded execution for '${node_name}' on '${host}'"
    if run_remote_bootstrap "${host}" "${remote_root}" "${node_name}" "${ROUNDS}" "${SLEEP_MS}" "${bin_path}"; then
      SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
      FAILURE_COUNT=$((FAILURE_COUNT + 1))
      fail "Remote validation failed for host '${host}' and node '${node_name}'"
    fi

    log_info "Collecting remote artifacts for '${node_name}'"
    collect_remote_artifacts "${host}" "${remote_root}" "${node_name}"
  done < "${SANITIZED_HOSTS_FILE}"

  log_stage "Writing summary and artifact index"
  write_summary_file
  write_artifact_index

  log_info "Distributed validation completed successfully"
  log_info "Artifacts available under ${ARTIFACT_DIR}"
}

main "$@"
