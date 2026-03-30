#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Execute mainnet readiness validation flows against an isolated AOXC runtime
#   home and fail closed on unmet operational prerequisites.
#
# Scope:
#   - Materialize an isolated readiness runtime home
#   - Bootstrap the selected production profile deterministically
#   - Execute mainnet readiness validation
#   - Execute full-surface readiness validation
#
# Security and Reliability Notes:
#   - Secrets must not be hardcoded in the script body
#   - Runtime state must remain isolated from default operator homes
#   - Binary resolution must be explicit and deterministic
#   - The command surface must not depend on shell-fragment execution
#
# Exit Codes:
#   0  Successful completion
#   2  Invalid configuration or missing dependency
#   3  AOXC binary resolution failure
#   4  Bootstrap failure
#   5  Readiness validation failure
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly DEFAULT_AOXC_HOME="${ROOT_DIR}/.artifacts/readiness-home"
readonly DEFAULT_PROFILE="mainnet"
readonly DEFAULT_VALIDATOR_NAME="validator-readiness"
readonly DEFAULT_BIND_HOST="0.0.0.0"

AOXC_HOME="${AOXC_HOME:-$DEFAULT_AOXC_HOME}"
PROFILE="${PROFILE:-$DEFAULT_PROFILE}"
VALIDATOR_NAME="${VALIDATOR_NAME:-$DEFAULT_VALIDATOR_NAME}"
BIND_HOST="${BIND_HOST:-$DEFAULT_BIND_HOST}"

log_info() {
  printf '[readiness][info] %s\n' "$*"
}

log_error() {
  printf '[readiness][error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"
  log_error "${message}"
  exit "${exit_code}"
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 2
}

resolve_bin_path() {
  # The binary resolution order is intentionally explicit to preserve
  # deterministic operator behavior across packaged and repository-local
  # runtime layouts.
  if [[ -n "${BIN_PATH:-}" ]]; then
    [[ -x "${BIN_PATH}" ]] || die "BIN_PATH is set but not executable: ${BIN_PATH}" 3
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

validate_configuration() {
  [[ -n "${AOXC_HOME}" ]] || die "AOXC_HOME must not be empty." 2
  [[ -n "${PROFILE}" ]] || die "PROFILE must not be empty." 2
  [[ -n "${VALIDATOR_NAME}" ]] || die "VALIDATOR_NAME must not be empty." 2
  [[ -n "${BIND_HOST}" ]] || die "BIND_HOST must not be empty." 2
  [[ -n "${AOXC_BOOTSTRAP_PASSWORD:-}" ]] || die "AOXC_BOOTSTRAP_PASSWORD must be provided via environment variable. Hardcoded credentials are not permitted." 2
}

ensure_runtime_home() {
  if [[ -e "${AOXC_HOME}" && ! -d "${AOXC_HOME}" ]]; then
    die "AOXC_HOME exists but is not a directory: ${AOXC_HOME}" 2
  fi

  mkdir -p "${AOXC_HOME}" || die "Unable to create AOXC_HOME: ${AOXC_HOME}" 2
}

run_bootstrap() {
  local bin_path="$1"

  log_info "Executing production bootstrap for profile '${PROFILE}'."

  if ! "${bin_path}" production-bootstrap \
    --password "${AOXC_BOOTSTRAP_PASSWORD}" \
    --profile "${PROFILE}" \
    --name "${VALIDATOR_NAME}" \
    --bind-host "${BIND_HOST}"; then
    die "production-bootstrap failed." 4
  fi
}

run_mainnet_readiness() {
  local bin_path="$1"

  log_info "Executing mainnet readiness validation."

  if ! "${bin_path}" mainnet-readiness --enforce --format json; then
    die "mainnet-readiness failed." 5
  fi
}

run_full_surface_readiness() {
  local bin_path="$1"

  log_info "Executing full-surface readiness validation."

  if ! "${bin_path}" full-surface-readiness --enforce --format json; then
    die "full-surface-readiness failed." 5
  fi
}

main() {
  local bin_path=''

  require_command mkdir
  validate_configuration
  ensure_runtime_home

  export AOXC_HOME

  if ! bin_path="$(resolve_bin_path)"; then
    die "Unable to locate an executable AOXC binary. Build or install it with: make package-bin" 3
  fi

  log_info "Resolved AOXC binary: ${bin_path}"
  log_info "Using isolated AOXC_HOME: ${AOXC_HOME}"

  run_bootstrap "${bin_path}"
  run_mainnet_readiness "${bin_path}"
  run_full_surface_readiness "${bin_path}"

  log_info "Readiness validation completed successfully."
}

main "$@"
