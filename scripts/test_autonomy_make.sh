#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Verify the AOXC autonomy-oriented Makefile and sqlite control-plane
#   integration under an isolated operator home.
#
# Scope:
#   - Execute dry-run validation for release matrix and publication targets
#   - Exercise the sqlite-backed autonomy control-plane lifecycle directly
#   - Validate the corresponding Makefile sqlite target plans through dry-run
#
# Safety Model:
#   - The script isolates HOME into a temporary directory
#   - All sqlite control-plane artifacts are written beneath that isolated root
#   - A cleanup trap removes temporary state on exit
#
# Exit Codes:
#   0  Successful completion
#   1  Invalid invocation or general verification failure
#   2  Missing required host dependency
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_NAME="$(basename "$0")"

TMP_HOME=''
ORIGINAL_HOME="${HOME:-}"

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

require_command() {
  local command_name="$1"

  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 2
}

cleanup() {
  local cleanup_exit_code=$?

  if [[ -n "${TMP_HOME}" && -d "${TMP_HOME}" ]]; then
    rm -rf "${TMP_HOME}"
  fi

  exit "${cleanup_exit_code}"
}

trap cleanup EXIT

run_step() {
  local step_title="$1"
  shift

  printf '\n==> %s\n' "${step_title}"
  "$@"
}

run_sqlite_lifecycle() {
  python3 ./scripts/autonomy_sqlite_ctl.py init
  python3 ./scripts/autonomy_sqlite_ctl.py set-env \
    --env mainnet \
    --desired-state running \
    --observed-state stopped \
    --note "preflight"
  python3 ./scripts/autonomy_sqlite_ctl.py event \
    --env mainnet \
    --action start \
    --status ok \
    --detail "manual verification"
  python3 ./scripts/autonomy_sqlite_ctl.py release \
    --version v0.0.0-test \
    --artifact /tmp/aoxc-test.tar.gz \
    --evidence /tmp/evidence.json
  python3 ./scripts/autonomy_sqlite_ctl.py status
  python3 ./scripts/autonomy_sqlite_ctl.py history --limit 5
}

main() {
  require_command make
  require_command python3
  require_command mktemp

  TMP_HOME="$(mktemp -d)"
  export HOME="${TMP_HOME}"
  export AOXC_DATA_ROOT="${HOME}/.AOXCData"

  printf '==> AOXC autonomy Make + sqlite verification\n'
  printf 'Repository root: %s\n' "$(pwd)"
  printf 'Original HOME: %s\n' "${ORIGINAL_HOME:-unset}"
  printf 'Isolated HOME: %s\n' "${HOME}"
  printf 'AOXC_DATA_ROOT: %s\n' "${AOXC_DATA_ROOT}"

  run_step "Step 1: Dry-run build matrix and publish targets" \
    make -n build-release-mainnet

  make -n build-release-testnet
  make -n build-release-devnet
  make -n build-release-matrix
  make -n publish-release

  run_step "Step 2: Exercise sqlite control-plane lifecycle" \
    run_sqlite_lifecycle

  run_step "Step 3: Dry-run sqlite Make targets" \
    make -n db-init-sqlite

  make -n db-status-sqlite
  make -n db-event-sqlite
  make -n db-release-sqlite
  make -n db-history-sqlite

  printf '\n==> Verification completed successfully\n'
}

main "$@"
