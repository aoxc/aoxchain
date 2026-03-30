#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Generate network production-closure evidence for AOXC validation and
#   release-readiness workflows.
#
# Scope:
#   - Execute multi-host validation
#   - Produce deterministic partition and fault-injection planning artifacts
#   - Produce snapshot and recovery planning artifacts
#   - Produce soak-validation and alerting artifacts
#
# Exit Codes:
#   0  Successful completion
#   2  Invalid invocation or configuration
#   3  Missing required dependency or executable
#   4  Scenario execution failure
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
readonly DEFAULT_SCENARIO="all"
readonly DEFAULT_ARTIFACT_DIR="${ROOT_DIR}/artifacts/network-production-closure"
readonly DEFAULT_DURATION_MINUTES=30
readonly DEFAULT_AOXC_BIN_CANDIDATE_1="${HOME}/.AOXCData/bin/aoxc"
readonly DEFAULT_AOXC_BIN_CANDIDATE_2="${ROOT_DIR}/bin/aoxc"

SCENARIO="${SCENARIO:-$DEFAULT_SCENARIO}"
ARTIFACT_DIR="${ARTIFACT_DIR:-$DEFAULT_ARTIFACT_DIR}"
DURATION_MINUTES="${DURATION_MINUTES:-$DEFAULT_DURATION_MINUTES}"

AOXC_BIN_PATH=''
SNAPSHOT_FILE=''
COMPAT_FILE=''
ALERT_FILE=''

log_info() {
  printf '[closure][info] %s\n' "$*"
}

log_error() {
  printf '[closure][error] %s\n' "$*" >&2
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
  $(basename "$0") [--scenario <all|multi-host|partition|fault|faults|recovery|snapshot|soak>] [--artifact-dir <path>]

Environment:
  SCENARIO          Scenario selector
  ARTIFACT_DIR      Output directory
  DURATION_MINUTES  Soak duration in minutes
  BIN_PATH          Optional explicit AOXC binary path
EOF
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 3
}

require_executable_file() {
  local file_path="$1"
  [[ -f "${file_path}" ]] || die "Required file does not exist: ${file_path}" 3
  [[ -x "${file_path}" ]] || die "Required file is not executable: ${file_path}" 3
}

validate_non_negative_integer() {
  local value="$1"
  local name="$2"
  [[ "${value}" =~ ^[0-9]+$ ]] || die "Invalid value for ${name}: '${value}'. A non-negative integer is required." 2
}

ensure_directory() {
  local dir_path="$1"

  if [[ -e "${dir_path}" && ! -d "${dir_path}" ]]; then
    die "Path exists but is not a directory: ${dir_path}" 2
  fi

  mkdir -p "${dir_path}" || die "Unable to create directory: ${dir_path}" 2
}

resolve_bin_path() {
  if [[ -n "${BIN_PATH:-}" ]]; then
    [[ -x "${BIN_PATH}" ]] || die "BIN_PATH is set but not executable: ${BIN_PATH}" 3
    printf '%s\n' "${BIN_PATH}"
    return 0
  fi

  if [[ -x "${DEFAULT_AOXC_BIN_CANDIDATE_1}" ]]; then
    printf '%s\n' "${DEFAULT_AOXC_BIN_CANDIDATE_1}"
    return 0
  fi

  if [[ -x "${DEFAULT_AOXC_BIN_CANDIDATE_2}" ]]; then
    printf '%s\n' "${DEFAULT_AOXC_BIN_CANDIDATE_2}"
    return 0
  fi

  if command -v aoxc >/dev/null 2>&1; then
    command -v aoxc
    return 0
  fi

  return 1
}

parse_args() {
  while (( $# > 0 )); do
    case "$1" in
      --scenario)
        shift
        [[ $# -gt 0 ]] || die "Missing value for --scenario" 2
        SCENARIO="$1"
        shift
        ;;
      --artifact-dir)
        shift
        [[ $# -gt 0 ]] || die "Missing value for --artifact-dir" 2
        ARTIFACT_DIR="$1"
        shift
        ;;
      --help|-h|help)
        print_usage
        exit 0
        ;;
      *)
        die "Unknown argument: $1" 2
        ;;
    esac
  done
}

initialize_paths() {
  ensure_directory "${ARTIFACT_DIR}"
  SNAPSHOT_FILE="${ARTIFACT_DIR}/snapshot-recovery.json"
  COMPAT_FILE="${ARTIFACT_DIR}/compat-matrix.json"
  ALERT_FILE="${ARTIFACT_DIR}/alert-rules.md"
}

run_multi_host() {
  local script_path="${SCRIPT_DIR}/multi_host_validation.sh"

  require_executable_file "${script_path}"
  log_info "Executing multi-host validation."
  "${script_path}"
}

run_partition_faults() {
  log_info "Generating partition and fault-injection planning artifacts."

  cat > "${ARTIFACT_DIR}/fault-injection-plan.md" <<EOF
# AOXC Fault Injection Plan

- scenario: partition
- scenario: delay
- scenario: drop
- scenario: restart
- scenario: timeout
- requirement: collect node-run output, health output, and operator timeline
- result file: ${ARTIFACT_DIR}/fault-injection-results.json
EOF

  cat > "${ARTIFACT_DIR}/fault-injection-results.json" <<EOF
{
  "status": "planned",
  "scenarios": [
    "partition",
    "delay",
    "drop",
    "restart",
    "timeout"
  ],
  "evidence_requirement": "attach host-level logs and remediation notes before release"
}
EOF
}

run_recovery() {
  [[ -n "${AOXC_BIN_PATH}" ]] || die "AOXC binary path is not initialized." 4

  log_info "Generating compatibility matrix and snapshot recovery plan."
  "${AOXC_BIN_PATH}" compat-matrix > "${COMPAT_FILE}"

  cat > "${SNAPSHOT_FILE}" <<EOF
{
  "status": "scripted",
  "steps": [
    "bootstrap source node",
    "export snapshot and journal",
    "wipe target runtime home",
    "import snapshot",
    "run node-health and runtime-status",
    "verify compat-matrix before rejoin"
  ],
  "verification": [
    "height monotonic after restore",
    "state root matches exported snapshot",
    "node rejoins without protocol mismatch"
  ]
}
EOF
}

run_soak() {
  [[ -n "${AOXC_BIN_PATH}" ]] || die "AOXC binary path is not initialized." 4

  log_info "Generating soak and alerting artifacts."
  "${AOXC_BIN_PATH}" production-audit --format json > "${ARTIFACT_DIR}/production-audit.json"
  "${AOXC_BIN_PATH}" runtime-status --format json > "${ARTIFACT_DIR}/runtime-status.json"

  cat > "${ARTIFACT_DIR}/soak-plan.json" <<EOF
{
  "status": "scripted",
  "duration_minutes": ${DURATION_MINUTES},
  "required_metrics": [
    "block progression",
    "peer count",
    "rpc errors",
    "process restarts",
    "readiness score"
  ],
  "artifacts": [
    "production-audit.json",
    "runtime-status.json",
    "telemetry-snapshot.json"
  ]
}
EOF

  cat > "${ARTIFACT_DIR}/telemetry-snapshot.json" <<'EOF'
{
  "status": "baseline",
  "alerts_required": [
    "validator stalled",
    "peer count below quorum",
    "rpc readiness score degraded",
    "snapshot recovery failed"
  ]
}
EOF

  cat > "${ARTIFACT_DIR}/aoxhub-rollout.json" <<'EOF'
{
  "status": "planned",
  "surfaces": [
    "mainnet",
    "testnet",
    "aoxhub"
  ],
  "requirements": [
    "hub api parity against mainnet/testnet baselines",
    "bridge and rpc health verified before promotion",
    "operator rollback contact and ownership recorded"
  ]
}
EOF

  cat > "${ALERT_FILE}" <<'EOF'
# AOXC Alert Rules

- Critical: no finalized progress within 3 block windows.
- Critical: peer count drops below configured quorum threshold.
- High: readiness score below 85 for more than 5 minutes.
- High: snapshot restore or state-sync rehearsal exits non-zero.
- Warning: RPC rate limit surge indicates abusive traffic or client regression.
EOF
}

main() {
  parse_args "$@"
  validate_non_negative_integer "${DURATION_MINUTES}" "DURATION_MINUTES"
  initialize_paths

  require_command mkdir
  require_command cat

  case "${SCENARIO}" in
    all)
      if ! AOXC_BIN_PATH="$(resolve_bin_path)"; then
        die "Unable to locate an executable AOXC binary. Build or install it with: make package-bin" 3
      fi
      run_multi_host
      run_partition_faults
      run_recovery
      run_soak
      ;;
    multi-host)
      run_multi_host
      ;;
    partition|fault|faults)
      run_partition_faults
      ;;
    recovery|snapshot)
      if ! AOXC_BIN_PATH="$(resolve_bin_path)"; then
        die "Unable to locate an executable AOXC binary. Build or install it with: make package-bin" 3
      fi
      run_recovery
      ;;
    soak)
      if ! AOXC_BIN_PATH="$(resolve_bin_path)"; then
        die "Unable to locate an executable AOXC binary. Build or install it with: make package-bin" 3
      fi
      run_soak
      ;;
    *)
      die "Unknown scenario: ${SCENARIO}" 2
      ;;
  esac

  printf '[done] network production closure artifacts in %s\n' "${ARTIFACT_DIR}"
}

main "$@"
