#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Provide deterministic repository quality gate execution across the canonical
#   AOXC single-runtime development surface.
#
# Operational Intent:
#   - Centralize quick, full, and release-oriented repository quality flows
#   - Fail closed on missing toolchain prerequisites
#   - Provide operator-readable execution banners and stable failure semantics
#   - Avoid environment fan-out or profile-specific orchestration
#
# Exit Codes:
#   0  Successful completion
#   2  Invalid invocation
#   3  Missing prerequisite command
#   4  Quality step failure
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly MODE="${1:-}"
readonly LOCK_MODE="--locked"

CARGO_BIN="${CARGO:-cargo}"

log_info() {
  printf '[quality-gate][info] %s\n' "$*"
}

log_error() {
  printf '[quality-gate][error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"

  log_error "${message}"
  exit "${exit_code}"
}

print_usage() {
  cat <<'USAGE'
Usage:
  ./scripts/quality_gate.sh <quick|full|release>
USAGE
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 3
}

enter_repo_root() {
  cd "${ROOT_DIR}"
}

run_step() {
  local label="$1"
  shift

  log_info "Running step: ${label}"
  "$@" || die "Quality step failed: ${label}" 4
}

ensure_audit_available() {
  if ! command -v cargo-audit >/dev/null 2>&1; then
    die "cargo-audit is required for this quality mode. Install it with: cargo install cargo-audit" 3
  fi
}

run_quick() {
  run_step "cargo fmt --all --check" "${CARGO_BIN}" fmt --all --check
  run_step "cargo check --workspace --locked" "${CARGO_BIN}" check --workspace "${LOCK_MODE}"
}

run_full() {
  run_step "cargo fmt --all --check" "${CARGO_BIN}" fmt --all --check
  run_step "cargo check --workspace --locked" "${CARGO_BIN}" check --workspace "${LOCK_MODE}"
  run_step "cargo test --workspace --exclude aoxchub --all-targets --locked" \
    "${CARGO_BIN}" test --workspace --exclude aoxchub --all-targets "${LOCK_MODE}"
  run_step "cargo clippy --workspace --exclude aoxchub --all-targets --all-features --locked -- -D warnings" \
    "${CARGO_BIN}" clippy --workspace --exclude aoxchub --all-targets --all-features "${LOCK_MODE}" -- -D warnings
  run_step "cargo check -p aoxchub --all-targets --locked" \
    "${CARGO_BIN}" check -p aoxchub --all-targets "${LOCK_MODE}"
  ensure_audit_available
  run_step "cargo audit" cargo audit
}

run_release() {
  run_step "cargo fmt --all --check" "${CARGO_BIN}" fmt --all --check
  run_step "cargo check --workspace --locked" "${CARGO_BIN}" check --workspace "${LOCK_MODE}"
  run_step "cargo test --workspace --exclude aoxchub --all-targets --locked" \
    "${CARGO_BIN}" test --workspace --exclude aoxchub --all-targets "${LOCK_MODE}"
  run_step "cargo check -p aoxchub --all-targets --locked" \
    "${CARGO_BIN}" check -p aoxchub --all-targets "${LOCK_MODE}"
  run_step "cargo build --release --workspace --bins --locked" \
    "${CARGO_BIN}" build --release --workspace --bins "${LOCK_MODE}"
  ensure_audit_available
  run_step "cargo audit" cargo audit
}

main() {
  [[ -n "${MODE}" ]] || {
    print_usage >&2
    exit 2
  }

  require_command "${CARGO_BIN}"
  require_command git
  enter_repo_root

  case "${MODE}" in
    quick)
      run_quick
      ;;
    full)
      run_full
      ;;
    release)
      run_release
      ;;
    --help|-h|help)
      print_usage
      ;;
    *)
      die "Unknown quality mode: ${MODE}" 2
      ;;
  esac

  log_info "Quality gate completed successfully for mode: ${MODE}"
}

main "$@"
