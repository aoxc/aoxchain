#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
ARTIFACT_DIR="${REPO_ROOT}/artifacts/aoxcvm-phase3"
MATRIX_FILE="${ARTIFACT_DIR}/determinism-matrix.json"
BENCH_FILE="${ARTIFACT_DIR}/gas-benchmark-envelope.json"

log() {
  echo "[aoxcvm-phase3-gate] $*"
}

run_step() {
  local label="$1"
  shift
  log "running: ${label}"
  "$@"
}

mkdir -p "${ARTIFACT_DIR}"
cd "${REPO_ROOT}"

run_step "cargo test -p aoxcvm --test phase3_release_closure" \
  cargo test -p aoxcvm --test phase3_release_closure

run_step "cargo test -p aoxcvm --test phase3_release_closure --release" \
  cargo test -p aoxcvm --test phase3_release_closure --release

run_step "cargo run -p aoxcvm --example determinism_probe" \
  bash -c 'cargo run -p aoxcvm --example determinism_probe > /tmp/aoxcvm_probe_debug.txt'
run_step "cargo run -p aoxcvm --example determinism_probe --release" \
  bash -c 'cargo run -p aoxcvm --example determinism_probe --release > /tmp/aoxcvm_probe_release.txt'

DEBUG_HASH="$(cat /tmp/aoxcvm_probe_debug.txt | tr -d '\n\r')"
RELEASE_HASH="$(cat /tmp/aoxcvm_probe_release.txt | tr -d '\n\r')"

if [[ -z "${DEBUG_HASH}" || -z "${RELEASE_HASH}" ]]; then
  echo "determinism probe output is empty" >&2
  exit 1
fi

if [[ "${DEBUG_HASH}" != "${RELEASE_HASH}" ]]; then
  echo "debug/release determinism mismatch: ${DEBUG_HASH} vs ${RELEASE_HASH}" >&2
  exit 1
fi

OS_NAME="$(uname -s)"
CPU_ARCH="$(uname -m)"
RUSTC_VERSION="$(rustc --version)"
GIT_SHA="$(git rev-parse HEAD)"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cat > "${MATRIX_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "probe": {
    "debug_hash": "${DEBUG_HASH}",
    "release_hash": "${RELEASE_HASH}",
    "parity": true
  },
  "host_fixture": {
    "os": "${OS_NAME}",
    "cpu_arch": "${CPU_ARCH}",
    "rustc": "${RUSTC_VERSION}"
  },
  "heterogeneous_validator_matrix": [
    {"os": "linux", "cpu": "x86_64", "status": "required"},
    {"os": "linux", "cpu": "aarch64", "status": "required"},
    {"os": "macos", "cpu": "aarch64", "status": "required"},
    {"os": "windows", "cpu": "x86_64", "status": "required"}
  ]
}
JSON

cat > "${BENCH_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "gas_table_justification": {
    "add": 3,
    "storage_write": 200,
    "pq_verify": 5000,
    "invariant": "add < storage_write < pq_verify"
  },
  "dos_cost_envelope": {
    "meter_limit": 10000,
    "expected_out_of_gas_on_pq_verify": true
  },
  "benchmark_diff_gate": {
    "policy": "manual-threshold-review",
    "required_on_release": true
  }
}
JSON

log "phase-3 gate completed; artifacts written to ${ARTIFACT_DIR}"
