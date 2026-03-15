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

case "${MODE}" in
  quick)
    run "Format check" cargo fmt --all -- --check
    run "Compile check" cargo check --workspace
    run "Tests" cargo test --workspace --no-fail-fast
    ;;
  full)
    run "Format check" cargo fmt --all -- --check
    run "Compile check" cargo check --workspace
    run "Clippy" cargo clippy --workspace --all-targets --all-features -- -D warnings
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
    exit 2
    ;;
esac

printf '\nQuality gate completed successfully in %q mode.\n' "${MODE}"
