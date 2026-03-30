#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Provide a lightweight autonomy-oriented control surface for blueprint
#   review, prerequisite diagnostics, and operator command profile output.
#
# Scope:
#   - Print the autonomous operations blueprint
#   - Validate local prerequisites for autonomy-oriented workflows
#   - Print the mainnet-only operator sidebar profile
#   - Print the integrated multi-network operator sidebar profile
#
# Exit Codes:
#   0  Successful completion
#   1  Prerequisite validation failure
#   2  Invalid invocation
#   9  Missing required host dependency
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_NAME="$(basename "$0")"
readonly SCRIPT_ROOT="scripts"

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

print_usage() {
  cat <<EOF
AOXC Autonomy Control Plane (experimental)

Usage:
  ${SCRIPT_NAME} <command>

Commands:
  plan            Print the implementation blueprint for autonomy-oriented operation
  doctor          Validate local prerequisites for autonomy-oriented workflows
  left-mainnet    Print the mainnet-only operator command profile
  right-multi     Print the integrated multi-network operator command profile
  help            Print this usage message
EOF
}

print_plan() {
  cat <<'PLAN'
=== AOXC Full Autonomous System Blueprint ===

1) Mainnet-first operating mode (left sidebar)
   - Wallet address lifecycle: create, import, export, and verify
   - Core transactional flows: transfer, stake, undelegate, and claim
   - Safety rails: fee estimation, preflight simulation, and policy checks
   - AI operator mode: intent -> reviewed transaction plan -> signed execution

2) Integrated multi-network mode (right sidebar)
   - Unified environment switch: mainnet, devnet, and testnet
   - Cross-environment command parity for operational primitives
   - Cross-chain interaction layer with route validation and replay protection
   - Central observability panel for health, latency, peers, and finality

3) Automation control loop
   - Policy engine validates every intent before execution
   - Scheduler handles periodic tasks such as status probes, restake, and evidence export
   - Risk controller can pause automation by environment or operation class

4) Delivery gates
   - Deterministic CLI surface with Make targets and scripts
   - Integration tests for transfer, stake, and bridge workflows across environments
   - Production readiness gate for the mainnet release candidate
PLAN
}

doctor_check_command() {
  local tool_name="$1"

  if command -v "${tool_name}" >/dev/null 2>&1; then
    printf '[ok] command available: %s\n' "${tool_name}"
    return 0
  fi

  printf '[missing] command not found: %s\n' "${tool_name}" >&2
  return 1
}

doctor_check_script() {
  local script_path="$1"

  if [[ -x "${script_path}" ]]; then
    printf '[ok] executable script: %s\n' "${script_path}"
    return 0
  fi

  if [[ -f "${script_path}" ]]; then
    printf '[warn] script exists but is not executable: %s\n' "${script_path}" >&2
    return 0
  fi

  printf '[missing] script not found: %s\n' "${script_path}" >&2
  return 1
}

run_doctor() {
  local failure_count=0
  local tool=''
  local script=''

  for tool in bash make cargo git; do
    if ! doctor_check_command "${tool}"; then
      failure_count=$((failure_count + 1))
    fi
  done

  for script in \
    "${SCRIPT_ROOT}/aoxc_easy.sh" \
    "${SCRIPT_ROOT}/network_env_daemon.sh" \
    "${SCRIPT_ROOT}/network_stack.sh"; do
    if ! doctor_check_script "${script}"; then
      failure_count=$((failure_count + 1))
    fi
  done

  if (( failure_count > 0 )); then
    die "Autonomy doctor completed with missing prerequisites." 1
  fi

  log_info "Autonomy doctor passed."
}

print_left_mainnet() {
  cat <<'LEFT'
=== Left Sidebar: Mainnet-Only Easy Operations ===

make ops-start-mainnet
make ops-status-mainnet
make ops-logs-mainnet
make ops-stop-mainnet

# Core economic actions via CLI
$AOXC_BIN_PATH wallet address create --network mainnet
$AOXC_BIN_PATH tx transfer --network mainnet <to> <amount>
$AOXC_BIN_PATH tx stake --network mainnet <validator> <amount>
LEFT
}

print_right_multi() {
  cat <<'RIGHT'
=== Right Sidebar: Integrated Full System ===

make ops-dashboard
make ops-status-mainnet
make ops-status-testnet
make ops-status-devnet
make net-dual-status

# Execution matrix (policy-gated)
$AOXC_BIN_PATH tx transfer --network {mainnet|testnet|devnet}
$AOXC_BIN_PATH tx stake --network {mainnet|testnet|devnet}
$AOXC_BIN_PATH bridge send --from <network> --to <network>
RIGHT
}

main() {
  local command="${1:-}"

  case "${command}" in
    plan)
      print_plan
      ;;
    doctor)
      run_doctor
      ;;
    left-mainnet)
      print_left_mainnet
      ;;
    right-multi)
      print_right_multi
      ;;
    help|-h|--help|"")
      print_usage
      ;;
    *)
      print_usage >&2
      die "Unknown command: ${command}" 2
      ;;
  esac
}

main "$@"
