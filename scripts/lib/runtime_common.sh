#!/usr/bin/env bash
# Shared helpers for AOXC script wrappers.
set -euo pipefail

readonly AOXC_SCRIPT_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly AOXC_REPO_ROOT="$(cd "${AOXC_SCRIPT_LIB_DIR}/../.." && pwd)"

log_info() {
  printf '[aoxc-script][info] %s\n' "$*"
}

log_error() {
  printf '[aoxc-script][error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="${2:-1}"
  log_error "${message}"
  exit "${exit_code}"
}

ensure_repo_root() {
  [[ -f "${AOXC_REPO_ROOT}/Makefile" ]] || die "Repository root is invalid: ${AOXC_REPO_ROOT}" 2
}

enter_repo_root() {
  ensure_repo_root
  cd "${AOXC_REPO_ROOT}"
}

require_non_empty() {
  local value="$1"
  local field="$2"
  [[ -n "${value// }" ]] || die "${field} must not be empty" 2
}

require_uint() {
  local value="$1"
  local field="$2"
  [[ "${value}" =~ ^[0-9]+$ ]] || die "${field} must be an unsigned integer: '${value}'" 2
}

require_non_placeholder_key() {
  local value="$1"
  local field="$2"
  if [[ "${value}" == replace_with_* ]]; then
    die "${field} uses placeholder value '${value}'. Provide a real key." 2
  fi
  [[ "${value}" =~ ^[0-9a-fA-F]{64,}$ ]] || die "${field} must be a hex public key" 2
}

run_make_target() {
  local target="$1"
  shift
  enter_repo_root
  log_info "Running make target: ${target}"
  make --no-print-directory "${target}" "$@"
}

require_executable() {
  local path="$1"
  [[ -x "${path}" ]] || die "Missing executable: ${path}" 2
}
