#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Execute a minimal local AOXC smoke flow by bootstrapping the node and
#   producing a single test transaction.
#
# Operational Intent:
#   - Resolve the AOXC binary from approved local locations
#   - Fail fast when the runtime binary is unavailable
#   - Execute the smallest meaningful local node workflow
#
# Exit Codes:
#   0  Successful completion
#   1  General operational failure or invalid runtime state
#   2  AOXC binary resolution failure
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

log_info() {
  printf '[info] %s\n' "$*"
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

resolve_bin_path() {
  # The binary resolution order is intentionally strict to preserve predictable
  # local execution semantics across packaged and repository-local runtimes.
  if [[ -n "${BIN_PATH:-}" ]]; then
    if [[ -x "${BIN_PATH}" ]]; then
      printf '%s\n' "${BIN_PATH}"
      return 0
    fi

    die "BIN_PATH is set but not executable: ${BIN_PATH}" 2
  fi

  if [[ -x "${HOME}/.AOXCData/bin/aoxc" ]]; then
    printf '%s\n' "${HOME}/.AOXCData/bin/aoxc"
    return 0
  fi

  if [[ -x "./bin/aoxc" ]]; then
    printf '%s\n' "./bin/aoxc"
    return 0
  fi

  return 1
}

main() {
  local bin_path=''

  if ! bin_path="$(resolve_bin_path)"; then
    die "Missing AOXC binary. Run: make package-bin. Expected paths: \$HOME/.AOXCData/bin/aoxc or ./bin/aoxc" 2
  fi

  log_info "Resolved AOXC binary: ${bin_path}"
  log_info "Executing local node bootstrap."
  "${bin_path}" node-bootstrap

  log_info "Producing local smoke transaction."
  "${bin_path}" produce-once --tx "local-smoke"

  log_info "Local smoke flow completed successfully."
}

main "$@"
