#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"

log() {
  echo "[testnet-readiness-gate] $*"
}

run_step() {
  local label="$1"
  shift
  log "running: ${label}"
  "$@"
}

cd "${REPO_ROOT}"

run_step "cargo fmt --all --check" cargo fmt --all --check
run_step "cargo clippy --workspace --all-targets --all-features -- -D warnings" \
  cargo clippy --workspace --all-targets --all-features -- -D warnings
run_step "cargo test -p aoxcai" cargo test -p aoxcai
run_step "cargo test -p tests" cargo test -p tests
run_step "python3 scripts/validation/network_identity_gate.py --env testnet --env mainnet --env devnet" \
  python3 scripts/validation/network_identity_gate.py --env testnet --env mainnet --env devnet
run_step "make testnet-gate" make --no-print-directory testnet-gate

log "all readiness checks passed"
