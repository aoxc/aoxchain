#!/usr/bin/env bash
set -euo pipefail

# AOXChain Production Quality Gate (v0.1.0-alpha)
#
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
    run "Tests (fast fail)" cargo test --workspace --no-fail-fast
    ;;
  full)
    run "Format check" cargo fmt --all -- --check
    run "Compile check" cargo check --workspace
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
    echo "Unknown mode: ${MODE}"
    echo "Usage: $0 [quick|full|release]"
    exit 2
    ;;
esac

echo "\nQuality gate completed successfully in '${MODE}' mode."
