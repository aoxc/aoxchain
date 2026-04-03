#!/usr/bin/env bash
# Shared helpers for AOXC script wrappers.
set -Eeuo pipefail
IFS=$'\n\t'

readonly AOXC_SCRIPT_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly AOXC_REPO_ROOT="$(cd "${AOXC_SCRIPT_LIB_DIR}/../.." && pwd)"

AOXC_SCRIPT_DRY_RUN="${AOXC_SCRIPT_DRY_RUN:-0}"
AOXC_SCRIPT_VERBOSE="${AOXC_SCRIPT_VERBOSE:-0}"

log_info() {
  printf '[aoxc-script][info] %s\n' "$*"
}

log_warn() {
  printf '[aoxc-script][warn] %s\n' "$*"
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

require_positive_uint() {
  local value="$1"
  local field="$2"
  require_uint "${value}" "${field}"
  (( value > 0 )) || die "${field} must be greater than zero" 2
}

require_non_placeholder_key() {
  local value="$1"
  local field="$2"
  if [[ "${value}" == replace_with_* ]]; then
    die "${field} uses placeholder value '${value}'. Provide a real key." 2
  fi
  [[ "${value}" =~ ^[0-9a-fA-F]{64,}$ ]] || die "${field} must be a hex public key" 2
}

run_cmd() {
  local -a cmd=("$@")

  if [[ "${AOXC_SCRIPT_DRY_RUN}" == "1" ]]; then
    printf '[aoxc-script][dry-run]'
    for arg in "${cmd[@]}"; do
      printf ' %q' "${arg}"
    done
    printf '\n'
    return 0
  fi

  if [[ "${AOXC_SCRIPT_VERBOSE}" == "1" ]]; then
    printf '[aoxc-script][exec]'
    for arg in "${cmd[@]}"; do
      printf ' %q' "${arg}"
    done
    printf '\n'
  fi

  "${cmd[@]}"
}

run_make_target() {
  local target="$1"
  shift
  enter_repo_root
  log_info "Running make target: ${target}"
  if [[ "${AOXC_SCRIPT_DRY_RUN}" == "1" ]]; then
    run_cmd make --no-print-directory "${target}" "$@"
  else
    make --no-print-directory "${target}" "$@"
  fi
}

require_executable() {
  local path="$1"
  [[ -x "${path}" ]] || die "Missing executable: ${path}" 2
}

print_common_flag_help() {
  cat <<'OUT'
Common flags:
  --dry-run     Print commands without executing
  --verbose     Print additional execution details
  --print-env   Print resolved variables before execution
  --help        Show script usage
OUT
}

parse_common_flags() {
  local -n _args_ref=$1
  local -a remaining=()
  local arg=""

  for arg in "${_args_ref[@]}"; do
    case "${arg}" in
      --dry-run)
        AOXC_SCRIPT_DRY_RUN=1
        ;;
      --verbose)
        AOXC_SCRIPT_VERBOSE=1
        ;;
      *)
        remaining+=("${arg}")
        ;;
    esac
  done

  _args_ref=("${remaining[@]}")
}
