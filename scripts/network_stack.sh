#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Coordinate dual-environment AOXC daemon operations across the canonical
#   `testnet` and `mainnet` runtime surfaces.
#
# Operational Model:
#   - Delegate per-environment lifecycle control to `network_env_daemon.sh`
#   - Execute ordered dual-stack actions for start, once, status, stop, and
#     restart flows
#   - Apply rollback semantics when a partial dual-start operation fails
#
# Exit Codes:
#   0   Successful completion
#   2   Invalid invocation
#   3   Unsupported command
#   4   Required daemon script is missing or not executable
#   10  Dual-start failed and rollback was initiated
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly DAEMON_SCRIPT="${SCRIPT_DIR}/network_env_daemon.sh"
readonly ENVIRONMENTS=("testnet" "mainnet")

log_info() {
  printf '[network-stack][info] %s\n' "$*"
}

log_warn() {
  printf '[network-stack][warn] %s\n' "$*" >&2
}

log_error() {
  printf '[network-stack][error] %s\n' "$*" >&2
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
  $(basename "$0") <start|once|status|stop|restart>
EOF
}

require_daemon_script() {
  [[ -f "${DAEMON_SCRIPT}" ]] || die "Required daemon script does not exist: ${DAEMON_SCRIPT}" 4
  [[ -x "${DAEMON_SCRIPT}" ]] || die "Required daemon script is not executable: ${DAEMON_SCRIPT}" 4
}

run_for_all() {
  local command="$1"
  local env=''

  for env in "${ENVIRONMENTS[@]}"; do
    log_info "Executing '${command}' for environment '${env}'."
    "${DAEMON_SCRIPT}" "${command}" "${env}"
  done
}

rollback_started_environments() {
  local started_environments=("$@")
  local env=''

  if (( ${#started_environments[@]} == 0 )); then
    return 0
  fi

  log_warn "Rolling back previously started environments."

  for env in "${started_environments[@]}"; do
    if ! "${DAEMON_SCRIPT}" stop "${env}"; then
      log_warn "Rollback stop failed for environment '${env}'. Manual operator intervention may be required."
    fi
  done
}

start_dual() {
  local started=()
  local env=''

  for env in "${ENVIRONMENTS[@]}"; do
    log_info "Starting environment '${env}'."

    if "${DAEMON_SCRIPT}" start "${env}"; then
      started+=("${env}")
      continue
    fi

    log_error "Failed to start environment '${env}'. Initiating rollback."
    rollback_started_environments "${started[@]}"
    exit 10
  done

  log_info "Dual stack is running across testnet and mainnet."
}

restart_dual() {
  log_info "Restarting dual stack."
  run_for_all stop
  start_dual
}

main() {
  local command="${1:-}"

  require_daemon_script

  if [[ -z "${command}" ]]; then
    print_usage >&2
    exit 2
  fi

  case "${command}" in
    start)
      start_dual
      ;;
    once|status|stop)
      run_for_all "${command}"
      ;;
    restart)
      restart_dual
      ;;
    --help|-h|help)
      print_usage
      ;;
    *)
      print_usage >&2
      die "Unsupported command: ${command}" 3
      ;;
  esac
}

main "$@"
