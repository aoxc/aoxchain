#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Verify that AOXC Makefile release packaging targets remain operationally
#   sound, deterministic, and ready for controlled execution.
#
# Scope:
#   - Resolve the current repository path model
#   - Validate configured release binary discovery
#   - Inspect packaging command plans through dry-run execution
#   - Optionally execute real packaging targets
#   - Optionally execute full workspace tests after native dependency checks
#
# Exit Codes:
#   0  Successful completion
#   1  Invalid invocation or general verification failure
#   2  Missing required host dependency
#   3  Missing required native development package
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_NAME="$(basename "$0")"
readonly REPO_ROOT="$(pwd)"

readonly USAGE="Usage: ${SCRIPT_NAME} [--run-build] [--run-workspace-tests] [--help]"

RUN_BUILD=0
RUN_WORKSPACE_TESTS=0

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
  printf '%s\n' "${USAGE}"
}

require_command() {
  local command_name="$1"

  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 2
}

parse_args() {
  local arg=''

  for arg in "$@"; do
    case "${arg}" in
      --run-build)
        RUN_BUILD=1
        ;;
      --run-workspace-tests)
        RUN_WORKSPACE_TESTS=1
        ;;
      --help|-h)
        print_usage
        exit 0
        ;;
      *)
        print_usage >&2
        die "Unknown argument: ${arg}" 1
        ;;
    esac
  done
}

verify_native_test_dependencies() {
  log_info "Validating native host dependencies required for workspace tests."

  require_command pkg-config

  if ! pkg-config --exists glib-2.0; then
    log_error "The glib-2.0 development package is not available on the host."
    log_error "Debian/Ubuntu hint: sudo apt-get install -y libglib2.0-dev pkg-config"
    exit 3
  fi
}

run_step() {
  local step_title="$1"
  shift

  printf '\n==> %s\n' "${step_title}"
  "$@"
}

main() {
  parse_args "$@"

  require_command make

  printf '==> AOXC Makefile release packaging verification\n'
  printf 'Repository root: %s\n' "${REPO_ROOT}"

  run_step "Step 1: Resolve path model" \
    make paths

  run_step "Step 2: Detect workspace binaries" \
    make release-binary-list

  run_step "Step 3: Validate packaging command plans (dry-run)" \
    make -n package-all-bin

  make -n package-versioned-bin
  make -n package-versioned-archive

  if (( RUN_BUILD == 1 )); then
    run_step "Step 4: Execute release packaging targets" \
      make package-versioned-bin

    make package-versioned-archive
  else
    log_info "Real packaging execution was not requested."
  fi

  if (( RUN_WORKSPACE_TESTS == 1 )); then
    printf '\n==> Step 5: Validate workspace test prerequisites\n'
    verify_native_test_dependencies

    log_info "Running full workspace tests."
    make test
  else
    log_info "Workspace test execution was not requested."
  fi

  printf '\n==> Verification flow completed successfully\n'
}

main "$@"
