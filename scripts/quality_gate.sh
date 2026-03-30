#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Enforce AOXC repository quality, security, and release-readiness gates for
#   local operator validation, CI workflows, and pre-release verification.
#
# Operational Modes:
#   quick   - Fast developer-oriented validation for formatting, compilation,
#             and core workspace tests
#   full    - Expanded validation including security audit, clippy, compile
#             checks, core workspace tests, and documentation tests
#   audit   - Release-oriented security and dependency validation with strict
#             compilation, build, artifact certification, and test coverage
#   release - Alias of audit mode for operational clarity in release workflows
#   desktop - Desktop surface compilation validation only
#
# Exit Codes:
#   0  Successful completion
#   1  General quality gate failure
#   2  Invalid invocation
#   9  Missing required host dependency
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_NAME="$(basename "$0")"
readonly DESKTOP_CRATE="aoxchub"

MODE="${1:-full}"

log_step() {
  printf '\n==> %s\n' "$1"
}

log_info() {
  printf '[info] %s\n' "$1"
}

log_warn() {
  printf '[warn] %s\n' "$1" >&2
}

log_error() {
  printf '[error] %s\n' "$1" >&2
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
  ${SCRIPT_NAME} {quick|full|audit|release|desktop}

Modes:
  quick    Run formatting, compile checks, and core workspace tests
  full     Run security audit, formatting, clippy, compile checks, tests, and doc tests
  audit    Run release-oriented audit, dependency validation, build, artifact certification, and tests
  release  Alias of audit mode
  desktop  Run desktop-only compilation checks
EOF
}

require_command() {
  local command_name="$1"

  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 9
}

ensure_cargo_tool() {
  local tool_name="$1"

  if command -v "${tool_name}" >/dev/null 2>&1; then
    return 0
  fi

  log_step "Installing missing cargo tool: ${tool_name}"
  cargo install "${tool_name}" --locked
}

run_checked() {
  local label="$1"
  shift

  log_step "${label}"

  if ! "$@"; then
    local exit_code=$?
    die "Step failed: ${label} (exit code: ${exit_code})" 1
  fi
}

run_quick_mode() {
  run_checked "Format Check" \
    cargo fmt --all --check

  run_checked "Compile Check (Core Workspace)" \
    cargo check --locked --workspace --exclude "${DESKTOP_CRATE}" --all-targets

  run_checked "Locked Unit Tests (Core Workspace)" \
    cargo test --locked --workspace --exclude "${DESKTOP_CRATE}" --no-fail-fast
}

run_full_mode() {
  ensure_cargo_tool cargo-audit

  run_checked "Security Audit" \
    cargo audit

  run_checked "Format Check" \
    cargo fmt --all --check

  run_checked "Linter (Clippy, Core Workspace)" \
    cargo clippy --workspace --exclude "${DESKTOP_CRATE}" --all-targets --all-features -- -D warnings

  run_checked "Compile Check (Core Workspace)" \
    cargo check --locked --workspace --exclude "${DESKTOP_CRATE}" --all-targets

  run_checked "Locked Comprehensive Tests (Core Workspace)" \
    cargo test --locked --workspace --exclude "${DESKTOP_CRATE}" --all-targets --no-fail-fast

  run_checked "Documentation Tests" \
    cargo test --doc
}

run_audit_mode() {
  ensure_cargo_tool cargo-audit
  ensure_cargo_tool cargo-deny

  run_checked "Vulnerability Audit" \
    cargo audit

  run_checked "License and Dependency Audit" \
    cargo deny check

  run_checked "Format Check" \
    cargo fmt --all --check

  run_checked "Compile Check (Release, Core Workspace)" \
    cargo check --locked --workspace --exclude "${DESKTOP_CRATE}" --all-targets --release

  run_checked "Clippy (Strict, Core Workspace)" \
    cargo clippy --workspace --exclude "${DESKTOP_CRATE}" --all-targets --all-features -- -D warnings

  run_checked "Build Production Binary" \
    cargo build --locked --release -p aoxcmd --bin aoxc

  run_checked "Release Artifact Certification" \
    ./scripts/release_artifact_certify.sh target/release/aoxc

  run_checked "Locked Production Test Suite (Core Workspace)" \
    cargo test --locked --workspace --exclude "${DESKTOP_CRATE}" --release --all-targets --no-fail-fast
}

run_desktop_mode() {
  run_checked "Compile Check (Desktop Tauri Surface)" \
    cargo check --locked -p "${DESKTOP_CRATE}" --all-targets
}

main() {
  require_command cargo

  case "${MODE}" in
    quick)
      run_quick_mode
      ;;
    full)
      run_full_mode
      ;;
    audit|release)
      run_audit_mode
      ;;
    desktop)
      run_desktop_mode
      ;;
    --help|-h|help)
      print_usage
      exit 0
      ;;
    *)
      print_usage >&2
      die "Unsupported mode: ${MODE}" 2
      ;;
  esac

  printf '\n[ok] Quality gate passed in "%s" mode.\n' "${MODE}"
}

main "$@"
