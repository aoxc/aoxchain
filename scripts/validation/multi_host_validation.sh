#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
#
# ------------------------------------------------------------------------------
# AOXC Distributed Validation Harness
# ------------------------------------------------------------------------------
# Purpose:
#   Execute a controlled distributed validation workflow across 3 to 5 remote
#   hosts using a deterministic fixture set.
#
# Operational intent:
#   - Synchronize a canonical fixture set to remote operators.
#   - Bootstrap deterministic node homes on each target host.
#   - Execute bounded node runs with deterministic transaction prefixes.
#   - Preserve per-node execution evidence under a local artifact directory.
#   - Avoid assumptions regarding root access, traffic shaping, or fault
#     injection primitives such as netem or iptables.
#
# Security and reliability posture:
#   - Fail closed on missing prerequisites and malformed host inventory.
#   - Avoid unsafe shell evaluation patterns.
#   - Keep remote command construction explicit and reviewable.
#   - Emit machine-reviewable logs and a human-oriented summary scaffold.
#
# Notes:
#   - The host inventory must contain one reachable host per line.
#   - Comment lines beginning with '#' and blank lines are ignored.
#   - The first five usable hosts are mapped deterministically to:
#       atlas, boreal, cypher, delta, ember

set -Eeuo pipefail
IFS=$'\n\t'

# ------------------------------------------------------------------------------
# Configuration
# ------------------------------------------------------------------------------
HOSTS_FILE="${HOSTS_FILE:-configs/deterministic-testnet/hosts.txt}"
HOSTS_TEMPLATE="${HOSTS_TEMPLATE:-configs/deterministic-testnet/hosts.txt.example}"
REMOTE_BASE="${REMOTE_BASE:-~/aoxc-distributed}"
FIXTURE_DIR="${FIXTURE_DIR:-configs/deterministic-testnet}"
ARTIFACT_DIR="${ARTIFACT_DIR:-artifacts/distributed-validation}"
AOXC_BIN="${AOXC_BIN:-cargo run -q -p aoxcmd --}"
ROUNDS="${ROUNDS:-5}"
SLEEP_MS="${SLEEP_MS:-250}"
SSH_OPTS="${SSH_OPTS:--o BatchMode=yes -o ConnectTimeout=10}"
RSYNC_OPTS="${RSYNC_OPTS:--az}"

# ------------------------------------------------------------------------------
# Deterministic node mapping
# ------------------------------------------------------------------------------
readonly NODE_NAMES=(
  "atlas"
  "boreal"
  "cypher"
  "delta"
  "ember"
)

# ------------------------------------------------------------------------------
# Runtime state
# ------------------------------------------------------------------------------
TMP_DIR="$(mktemp -d)"
SANITIZED_HOSTS_FILE="${TMP_DIR}/hosts.cleaned"
RUN_STARTED_AT_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
SUMMARY_FILE="${ARTIFACT_DIR}/summary.txt"
REPORT_FILE="${ARTIFACT_DIR}/distributed-validation-report.md"
INDEX_FILE="${ARTIFACT_DIR}/artifact-index.json"
FAILURE_FILE="${ARTIFACT_DIR}/failure.log"

SUCCESS_COUNT=0
FAILURE_COUNT=0

# ------------------------------------------------------------------------------
# Logging and failure handling
# ------------------------------------------------------------------------------
log() {
  printf '[info] %s\n' "$*"
}

stage() {
  printf '[stage] %s\n' "$*"
}

warn() {
  printf '[warn] %s\n' "$*" >&2
}

fail() {
  printf '[error] %s\n' "$*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

require_file() {
  [[ -f "$1" ]] || fail "Missing required file: $1"
}

require_dir() {
  [[ -d "$1" ]] || fail "Missing required directory: $1"
}

cleanup() {
  rm -rf "${TMP_DIR}"
}

on_error() {
  local exit_code="$1"
  local failed_line="$2"

  mkdir -p "${ARTIFACT_DIR}"

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

  printf '[error] Distributed validation failed at line %s with exit code %s\n' "${failed_line}" "${exit_code}" >&2
}

trap 'on_error "$?" "$LINENO"' ERR
trap cleanup EXIT

# ------------------------------------------------------------------------------
# Validation helpers
# ------------------------------------------------------------------------------
validate_positive_integer() {
  local value="$1"
  local name="$2"

  [[ "${value}" =~ ^[0-9]+$ ]] || fail "${name} must be a non-negative integer"
}

sanitize_hosts_file() {
  grep -v '^[[:space:]]*#' "${HOSTS_FILE}" | sed '/^[[:space:]]*$$/d' > "${SANITIZED_HOSTS_FILE}"
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
  python3 - "${INDEX_FILE}" "${ARTIFACT_DIR}" "${RUN_STARTED_AT_UTC}" "${SUCCESS_COUNT}" "${FAILURE_COUNT}" <<'PY'
import json
import sys
from pathlib import Path

output_path = Path(sys.argv[1])
artifact_dir = Path(sys.argv[2])
generated_at = sys.argv[3]
success_count = int(sys.argv[4])
failure_count = int(sys.argv[5])

artifacts = []
for path in sorted(artifact_dir.glob("*")):
    if path.is_file():
        artifacts.append(
            {
                "name": path.name,
                "path": str(path),
                "size_bytes": path.stat().st_size,
            }
        )

document = {
    "generated_at_utc": generated_at,
    "artifact_dir": str(artifact_dir),
    "success_count": success_count,
    "failure_count": failure_count,
    "artifacts": artifacts,
}

with output_path.open("w", encoding="utf-8") as handle:
    json.dump(document, handle, indent=2)
    handle.write("\n")
PY
}

run_remote_command() {
  local host="$1"
  local command="$2"
  ssh ${SSH_OPTS} "${host}" "${command}"
}

sync_fixture_to_host() {
  local host="$1"
  stage "Synchronizing deterministic fixture to ${host}"
  rsync ${RSYNC_OPTS} "${FIXTURE_DIR}/" "${host}:${REMOTE_BASE}/fixture/"
}

bootstrap_node_on_host() {
  local host="$1"
  local node_name="$2"
  local output_file="$3"

  stage "Bootstrapping ${node_name} on ${host}"
  run_remote_command \
    "${host}" \
    "mkdir -p ${REMOTE_BASE} && cd ${REMOTE_BASE} && ${AOXC_BIN} node-bootstrap --home fixture/homes/${node_name}" \
    | tee "${output_file}"
}

run_node_on_host() {
  local host="$1"
  local node_name="$2"
  local output_file="$3"
  local tx_prefix

  tx_prefix="$(printf '%s' "${node_name}" | tr '[:lower:]' '[:upper:]')-DIST"

  stage "Running bounded execution for ${node_name} on ${host}"
  run_remote_command \
    "${host}" \
    "cd ${REMOTE_BASE} && ${AOXC_BIN} node-run --home fixture/homes/${node_name} --rounds ${ROUNDS} --sleep-ms ${SLEEP_MS} --tx-prefix ${tx_prefix}" \
    | tee "${output_file}"
}

record_host_result() {
  local host="$1"
  local node_name="$2"
  local status="$3"

  printf 'host=%s node=%s status=%s\n' "${host}" "${node_name}" "${status}" >> "${SUMMARY_FILE}"
}

# ------------------------------------------------------------------------------
# Dependency and input validation
# ------------------------------------------------------------------------------
require_command grep
require_command sed
require_command ssh
require_command rsync
require_command tee
require_command mktemp
require_command python3

require_dir "${FIXTURE_DIR}"
mkdir -p "${ARTIFACT_DIR}"
require_dir "${ARTIFACT_DIR}"

validate_positive_integer "${ROUNDS}" "ROUNDS"
validate_positive_integer "${SLEEP_MS}" "SLEEP_MS"

if [[ ! -f "${HOSTS_FILE}" ]]; then
  printf '[error] Missing host file: %s\n' "${HOSTS_FILE}" >&2
  if [[ -f "${HOSTS_TEMPLATE}" ]]; then
    printf '[hint] Copy %s to %s and replace placeholders with 3 to 5 reachable hosts.\n' "${HOSTS_TEMPLATE}" "${HOSTS_FILE}" >&2
  else
    printf '[hint] Create %s with one reachable host per line for 3 to 5 hosts.\n' "${HOSTS_FILE}" >&2
  fi
  exit 2
fi

sanitize_hosts_file
mapfile -t HOSTS < "${SANITIZED_HOSTS_FILE}"

validate_host_count "${#HOSTS[@]}"

for host in "${HOSTS[@]}"; do
  validate_host_line "${host}"
done

write_summary_file
write_report_scaffold

# ------------------------------------------------------------------------------
# Distributed validation workflow
# ------------------------------------------------------------------------------
for i in "${!HOSTS[@]}"; do
  host="${HOSTS[$i]}"
  node_name="${NODE_NAMES[$i]}"

  [[ -n "${node_name}" ]] || fail "Node mapping is unavailable for host index ${i}"

  bootstrap_artifact="${ARTIFACT_DIR}/${node_name}-bootstrap.json"
  run_artifact="${ARTIFACT_DIR}/${node_name}-run.json"

  sync_fixture_to_host "${host}"

  if bootstrap_node_on_host "${host}" "${node_name}" "${bootstrap_artifact}" && \
     run_node_on_host "${host}" "${node_name}" "${run_artifact}"; then
    SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    record_host_result "${host}" "${node_name}" "success"
  else
    FAILURE_COUNT=$((FAILURE_COUNT + 1))
    record_host_result "${host}" "${node_name}" "failure"
    fail "Distributed validation failed for node ${node_name} on host ${host}"
  fi
done

write_summary_file
write_artifact_index

log "Distributed validation artifacts written under ${ARTIFACT_DIR}"
