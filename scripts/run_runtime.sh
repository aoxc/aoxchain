#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Execute the smallest meaningful AOXC single-runtime smoke flow by delegating
#   to the canonical runtime daemon surface.
#
# Operational Intent:
#   - Resolve the canonical runtime daemon helper
#   - Preserve one authoritative lifecycle surface for bootstrap and execution
#   - Fail fast on missing helper scripts or invalid repository layout
#
# Exit Codes:
#   0  Successful completion
#   1  General operational failure or invalid runtime state
#   2  Runtime daemon resolution failure
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly RUNTIME_DAEMON_SCRIPT="${SCRIPT_DIR}/runtime_daemon.sh"

log_info() {
  printf '[run-runtime][info] %s\n' "$*"
}

log_error() {
  printf '[run-runtime][error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"
  log_error "${message}"
  exit "${exit_code}"
}

main() {
  [[ -f "${RUNTIME_DAEMON_SCRIPT}" ]] || die "Missing runtime daemon script: ${RUNTIME_DAEMON_SCRIPT}" 2

  if [[ ! -x "${RUNTIME_DAEMON_SCRIPT}" ]]; then
    chmod +x "${RUNTIME_DAEMON_SCRIPT}" || die "Failed to mark runtime daemon script as executable: ${RUNTIME_DAEMON_SCRIPT}" 2
  fi

  log_info "Delegating single-runtime smoke flow to runtime daemon."
  exec "${RUNTIME_DAEMON_SCRIPT}" once
}

main "$@"
