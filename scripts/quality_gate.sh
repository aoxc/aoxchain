#!/usr/bin/env bash
set -euo pipefail

# AOXChain Production Quality Gate (v0.1.0-alpha)
#
# Deterministic and CI-friendly validation entrypoint.
# Supports quick/full/release modes.

MODE="${1:-full}"

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "[error] required command not found: $cmd" >&2
    exit 127
  fi
}

run() {
  local label="$1"
  shift
  printf '\n==> %s\n' "$label"
  "$@"
}

require_cmd cargo

# This script is intentionally deterministic and CI-friendly. It can be used in
# local development, CI pipelines, and pre-release validation.

MODE="${1:-full}"

run() {
  local label="$1"
  shift
  echo "\n==> ${label}"
  "$@"
}

case "${MODE}" in
  quick)
    run "Format check" cargo fmt --all -- --check
    run "Compile check" cargo check --workspace
    run "Tests" cargo test --workspace --no-fail-fast

    run "Tests (fast fail)" cargo test --workspace --no-fail-fast
    ;;
  full)
    run "Format check" cargo fmt --all -- --check
    run "Compile check" cargo check --workspace
    run "Clippy" cargo clippy --workspace --all-targets --all-features -- -D warnings
    run "Clippy (warnings as warnings)" cargo clippy --workspace --all-targets --all-features
    run "Tests" cargo test --workspace --no-fail-fast
    ;;
  release)
    run "Format check" cargo fmt --all -- --check
    run "Compile check (release)" cargo check --workspace --release
    run "Build release CLI" cargo build --release -p aoxcmd --bin aoxc
    run "Tests (release)" cargo test --workspace --release --no-fail-fast
    ;;
  *)
    echo "Unknown mode: ${MODE}" >&2
    echo "Usage: $0 [quick|full|release]" >&2
    echo "Unknown mode: ${MODE}"
    echo "Usage: $0 [quick|full|release]"
    exit 2
    ;;
esac

printf '\nQuality gate completed successfully in %q mode.\n' "${MODE}"
echo "\nQuality gate completed successfully in '${MODE}' mode."
